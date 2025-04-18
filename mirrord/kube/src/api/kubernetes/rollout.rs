use std::borrow::Cow;

use k8s_openapi::{
    api::core::v1::PodTemplateSpec,
    apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta},
    ListableResource, Metadata, NamespaceResourceScope, Resource,
};
use kube::Client;
use serde::{Deserialize, Serialize};

use crate::error::KubeApiError;

pub mod serialization;
pub mod spec_template;
pub mod workload_ref;

use self::{spec_template::RolloutSpecTemplate, workload_ref::WorkloadRef};

#[derive(Clone, Debug)]
pub struct Rollout {
    pub metadata: ObjectMeta,
    pub spec: Option<RolloutSpec>,
    pub status: Option<RolloutStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RolloutStatus {
    pub available_replicas: Option<i32>,
    /// Looks like this is a string for some reason:
    /// [rollouts/v1alpha1/types.go](https://github.com/argoproj/argo-rollouts/blob/4f1edbe9332b93d8aaf1d8f34239da6f952b8a93/pkg/apis/rollouts/v1alpha1/types.go#L922)
    pub observed_generation: Option<String>,
    pub pause_conditions: Option<serde_json::Value>,
}

/// Argo [`Rollout`]s provide `Pod` template in one of two ways:
/// 1. Inline (`template` field).
/// 2. Via a reference to some Kubernetes workload (`workloadRef` field).
///
/// See [Rollout spec](https://argoproj.github.io/argo-rollouts/features/specification/) for reference.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RolloutSpec {
    pub replicas: Option<i32>,
    pub selector: Option<LabelSelector>,
    pub template: Option<RolloutSpecTemplate>,
    pub workload_ref: Option<WorkloadRef>,
    pub strategy: Option<serde_json::Value>,
    pub analysis: Option<serde_json::Value>,
    pub min_ready_seconds: Option<i32>,
    pub revision_history_limit: Option<i32>,
    pub paused: Option<bool>,
    pub progress_deadline_seconds: Option<i32>,
    pub progress_deadline_abort: Option<bool>,
    pub restart_at: Option<String>,
    pub rollback_window: Option<serde_json::Value>,
}

impl Rollout {
    /// Get the pod template spec out of a rollout spec.
    /// Make requests to k8s if necessary when the template is not directly included in the
    /// rollout spec ([`RolloutSpec::template`]), but only referenced via a workload_ref
    /// ([`RolloutSpec::workload_ref`]).
    pub async fn get_pod_template<'a>(
        &'a self,
        client: &Client,
    ) -> Result<Cow<'a, PodTemplateSpec>, KubeApiError> {
        let spec = self
            .spec
            .as_ref()
            .ok_or_else(|| KubeApiError::missing_field(self, ".spec"))?;

        match spec {
            RolloutSpec {
                template: Some(..),
                workload_ref: Some(..),
                ..
            } => Err(KubeApiError::invalid_state(
                self,
                "both `.spec.template` and `.spec.workladRef` fields are filled",
            )),

            RolloutSpec {
                template: None,
                workload_ref: None,
                ..
            } => Err(KubeApiError::invalid_state(
                self,
                "both `.spec.template` and `.spec.workloadRef` fields are empty",
            )),

            RolloutSpec {
                template: Some(template),
                ..
            } => Ok(Cow::Borrowed(template.as_ref())),

            RolloutSpec {
                workload_ref: Some(workload_ref),
                ..
            } => workload_ref
                .get_pod_template(client, self.metadata.namespace.as_deref())
                .await?
                .ok_or_else(|| {
                    KubeApiError::invalid_state(
                        self,
                        format_args!(
                            "field `.spec.workloadRef` refers to an unknown resource `{}/{}`",
                            workload_ref.api_version, workload_ref.kind
                        ),
                    )
                })
                .map(Cow::Owned),
        }
    }

    /// Extracts [`LabelSelector`] from the [`Rollout`] or from the inner [`WorkloadRef`]
    ///
    /// Unlike `RuntimeDataFromLabels` trait the selector may also exist inside of the workloadRef
    /// target thus we need an async variant to fetch with a client.
    pub async fn get_match_labels<'a>(
        &'a self,
        client: &Client,
    ) -> Result<Cow<'a, LabelSelector>, KubeApiError> {
        let spec = self
            .spec
            .as_ref()
            .ok_or_else(|| KubeApiError::missing_field(self, ".spec"))?;

        match spec {
            RolloutSpec {
                selector: None,
                workload_ref: None,
                ..
            } => Err(KubeApiError::invalid_state(
                self,
                "both `.spec.selector` and `.spec.workloadRef` fields are empty",
            )),

            // Selector from the rollout spec overwrites the selector from the referenced workload
            // (if any).
            RolloutSpec {
                selector: Some(selector),
                ..
            } => Ok(Cow::Borrowed(selector)),

            RolloutSpec {
                workload_ref: Some(workload_ref),
                ..
            } => workload_ref
                .get_match_labels(client, self.metadata.namespace.as_deref())
                .await?
                .ok_or_else(|| {
                    KubeApiError::invalid_state(
                        self,
                        format_args!(
                            "field `.spec.workloadRef` refers to an unknown resource `{}/{}`",
                            workload_ref.api_version, workload_ref.kind
                        ),
                    )
                })
                .map(Cow::Owned),
        }
    }
}

impl Resource for Rollout {
    const API_VERSION: &'static str = "argoproj.io/v1alpha1";
    const GROUP: &'static str = "argoproj.io";
    const KIND: &'static str = "Rollout";
    const VERSION: &'static str = "v1alpha1";
    const URL_PATH_SEGMENT: &'static str = "rollouts";
    type Scope = NamespaceResourceScope;
}

impl ListableResource for Rollout {
    const LIST_KIND: &'static str = "RolloutList";
}

impl Metadata for Rollout {
    type Ty = ObjectMeta;

    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
    }
}

#[cfg(test)]
pub mod test {
    #[test]
    fn test_rollout() {
        let raw_json = r#"{
  "apiVersion": "argoproj.io/v1alpha1",
  "kind": "Rollout",
  "metadata": {
    "annotations": {
      "kubectl.kubernetes.io/last-applied-configuration": "{}\n",
      "rollout.argoproj.io/revision": "19"
    },
    "creationTimestamp": "2024-04-17T19:35:25Z",
    "generation": 41,
    "labels": {
      "app.kubernetes.io/instance": "test-web-stage",
      "app.kubernetes.io/managed-by": "Helm",
      "app.kubernetes.io/name": "test-web",
      "app.kubernetes.io/version": "1.929.0",
      "argocd.argoproj.io/instance": "gl-test-web-stage-e2",
      "helm.sh/chart": "test-web-1.929.0",
      "tags.datadoghq.com/env": "staging",
      "tags.datadoghq.com/test-web.env": "staging",
      "tags.datadoghq.com/test-web.service": "test-web",
      "tags.datadoghq.com/test-web.version": "1.929.0",
      "tags.datadoghq.com/service": "test-web",
      "tags.datadoghq.com/version": "1.929.0"
    },
    "managedFields": [
      {
        "apiVersion": "argoproj.io/v1alpha1",
        "fieldsType": "FieldsV1",
        "fieldsV1": {
          "f:spec": {
            "f:replicas": {}
          }
        },
        "manager": "kube-controller-manager",
        "operation": "Update",
        "subresource": "scale"
      },
      {
        "apiVersion": "argoproj.io/v1alpha1",
        "fieldsType": "FieldsV1",
        "fieldsV1": {
          "f:metadata": {
            "f:annotations": {
              "f:rollout.argoproj.io/revision": {}
            }
          }
        },
        "manager": "rollouts-controller",
        "operation": "Update",
        "time": "2024-06-10T16:08:41Z"
      },
      {
        "apiVersion": "argoproj.io/v1alpha1",
        "fieldsType": "FieldsV1",
        "fieldsV1": {
          "f:metadata": {
            "f:annotations": {
              ".": {},
              "f:kubectl.kubernetes.io/last-applied-configuration": {}
            },
            "f:labels": {
              ".": {},
              "f:app.kubernetes.io/instance": {},
              "f:app.kubernetes.io/managed-by": {},
              "f:app.kubernetes.io/name": {},
              "f:app.kubernetes.io/version": {},
              "f:argocd.argoproj.io/instance": {},
              "f:helm.sh/chart": {},
              "f:tags.datadoghq.com/env": {},
              "f:tags.datadoghq.com/test-web.env": {},
              "f:tags.datadoghq.com/test-web.service": {},
              "f:tags.datadoghq.com/test-web.version": {},
              "f:tags.datadoghq.com/service": {},
              "f:tags.datadoghq.com/version": {}
            }
          },
          "f:spec": {
            ".": {},
            "f:revisionHistoryLimit": {},
            "f:selector": {
              ".": {},
              "f:matchLabels": {
                ".": {},
                "f:app.kubernetes.io/instance": {},
                "f:app.kubernetes.io/name": {}
              }
            },
            "f:strategy": {
              ".": {},
              "f:canary": {
                ".": {},
                "f:analysis": {
                  ".": {},
                  "f:templates": {}
                },
                "f:maxSurge": {},
                "f:maxUnavailable": {},
                "f:steps": {}
              }
            },
            "f:template": {
              ".": {},
              "f:metadata": {
                ".": {},
                "f:annotations": {
                  ".": {},
                  "f:ad.datadoghq.com/test-web.check_names": {},
                  "f:ad.datadoghq.com/test-web.init_configs": {},
                  "f:ad.datadoghq.com/test-web.instances": {},
                  "f:ad.datadoghq.com/test-web.tags": {}
                },
                "f:labels": {
                  ".": {},
                  "f:app.kubernetes.io/instance": {},
                  "f:app.kubernetes.io/name": {},
                  "f:tags.datadoghq.com/env": {},
                  "f:tags.datadoghq.com/test-web.env": {},
                  "f:tags.datadoghq.com/test-web.service": {},
                  "f:tags.datadoghq.com/test-web.version": {},
                  "f:tags.datadoghq.com/service": {},
                  "f:tags.datadoghq.com/version": {}
                }
              },
              "f:spec": {
                ".": {},
                "f:affinity": {
                  ".": {},
                  "f:podAntiAffinity": {
                    ".": {},
                    "f:preferredDuringSchedulingIgnoredDuringExecution": {}
                  }
                },
                "f:containers": {},
                "f:imagePullSecrets": {},
                "f:serviceAccountName": {}
              }
            }
          }
        },
        "manager": "argocd-controller",
        "operation": "Update",
        "time": "2024-06-10T16:09:32Z"
      },
      {
        "apiVersion": "argoproj.io/v1alpha1",
        "fieldsType": "FieldsV1",
        "fieldsV1": {
          "f:status": {
            ".": {},
            "f:HPAReplicas": {},
            "f:availableReplicas": {},
            "f:blueGreen": {},
            "f:canary": {},
            "f:conditions": {},
            "f:currentPodHash": {},
            "f:currentStepHash": {},
            "f:currentStepIndex": {},
            "f:observedGeneration": {},
            "f:phase": {},
            "f:readyReplicas": {},
            "f:replicas": {},
            "f:selector": {},
            "f:stableRS": {},
            "f:updatedReplicas": {}
          }
        },
        "manager": "rollouts-controller",
        "operation": "Update",
        "subresource": "status",
        "time": "2024-06-11T11:30:40Z"
      }
    ],
    "name": "test-web",
    "namespace": "stage",
    "resourceVersion": "1861976373",
    "uid": "8eba1f7b-9e3b-4cb7-93fe-ce6a26c27686",
    "selfLink": "/apis/argoproj.io/v1alpha1/namespaces/stage/rollouts/test-web"
  },
  "status": {
    "HPAReplicas": 2,
    "availableReplicas": 2,
    "blueGreen": {},
    "canary": {},
    "conditions": [
      {
        "lastTransitionTime": "2024-06-10T16:16:21Z",
        "lastUpdateTime": "2024-06-10T16:16:21Z",
        "message": "RolloutCompleted",
        "reason": "RolloutCompleted",
        "status": "True",
        "type": "Completed"
      },
      {
        "lastTransitionTime": "2024-06-10T16:16:21Z",
        "lastUpdateTime": "2024-06-10T16:16:21Z",
        "message": "Rollout is paused",
        "reason": "RolloutPaused",
        "status": "False",
        "type": "Paused"
      },
      {
        "lastTransitionTime": "2024-06-11T11:30:40Z",
        "lastUpdateTime": "2024-06-11T11:30:40Z",
        "message": "Rollout is healthy",
        "reason": "RolloutHealthy",
        "status": "True",
        "type": "Healthy"
      },
      {
        "lastTransitionTime": "2024-06-10T16:16:21Z",
        "lastUpdateTime": "2024-06-11T11:30:40Z",
        "message": "ReplicaSet \"test-web-7ff8567587\" has successfully progressed.",
        "reason": "NewReplicaSetAvailable",
        "status": "True",
        "type": "Progressing"
      },
      {
        "lastTransitionTime": "2024-06-11T11:30:40Z",
        "lastUpdateTime": "2024-06-11T11:30:40Z",
        "message": "Rollout has minimum availability",
        "reason": "AvailableReason",
        "status": "True",
        "type": "Available"
      }
    ],
    "currentPodHash": "7ff8567587",
    "currentStepHash": "f847f885c",
    "currentStepIndex": 6,
    "observedGeneration": "41",
    "phase": "Healthy",
    "readyReplicas": 2,
    "replicas": 2,
    "selector": "app.kubernetes.io/instance=test-web-stage,app.kubernetes.io/name=test-web",
    "stableRS": "7ff8567587",
    "updatedReplicas": 2
  },
  "spec": {
    "replicas": 2,
    "revisionHistoryLimit": 1,
    "selector": {
      "matchLabels": {
        "app.kubernetes.io/instance": "test-web-stage",
        "app.kubernetes.io/name": "test-web"
      }
    },
    "strategy": {
      "canary": {
        "analysis": {
          "templates": [
            {
              "templateName": "test-web-express-http-success-rate"
            }
          ]
        },
        "maxSurge": "5%",
        "maxUnavailable": "5%",
        "steps": [
          {
            "setWeight": 25
          },
          {
            "pause": {
              "duration": "2m"
            }
          },
          {
            "setWeight": 50
          },
          {
            "pause": {
              "duration": "2m"
            }
          },
          {
            "setWeight": 100
          },
          {
            "pause": {
              "duration": "2m"
            }
          }
        ]
      }
    },
    "template": {
      "metadata": {
        "labels": {
          "app.kubernetes.io/instance": "test-web-stage",
          "app.kubernetes.io/name": "test-web",
          "tags.datadoghq.com/env": "staging",
          "tags.datadoghq.com/test-web.env": "staging",
          "tags.datadoghq.com/test-web.service": "test-web",
          "tags.datadoghq.com/test-web.version": "1.929.0",
          "tags.datadoghq.com/service": "test-web",
          "tags.datadoghq.com/version": "1.929.0"
        }
      },
      "spec": {
        "affinity": {
          "podAntiAffinity": {
            "preferredDuringSchedulingIgnoredDuringExecution": [
              {
                "podAffinityTerm": {
                  "labelSelector": {
                    "matchExpressions": [
                      {
                        "key": "app.kubernetes.io/name",
                        "operator": "In",
                        "values": [
                          "test-web"
                        ]
                      }
                    ]
                  },
                  "topologyKey": "failure-domain.beta.kubernetes.io/zone"
                },
                "weight": 100
              }
            ]
          }
        },
        "containers": [
          {
            "image": "shbd-docker.jfrog.io/test-web:1.929.0",
            "imagePullPolicy": "IfNotPresent",
            "livenessProbe": {
              "httpGet": {
                "path": "/healthz",
                "port": "http"
              },
              "initialDelaySeconds": 30,
              "periodSeconds": 10
            },
            "name": "test-web",
            "ports": [
              {
                "containerPort": 3000,
                "name": "http",
                "protocol": "TCP"
              }
            ],
            "readinessProbe": {
              "httpGet": {
                "path": "/healthz",
                "port": "http"
              },
              "initialDelaySeconds": 30,
              "periodSeconds": 5
            },
            "resources": {
              "limits": {
                "cpu": 1,
                "memory": "1024Mi"
              },
              "requests": {
                "cpu": "500m",
                "memory": "512Mi"
              }
            },
            "startupProbe": {
              "httpGet": {
                "path": "/healthz",
                "port": "http"
              },
              "initialDelaySeconds": 10,
              "periodSeconds": 10
            }
          }
        ],
        "imagePullSecrets": [
          {
            "name": "pp-docker"
          }
        ],
        "serviceAccountName": "test-web"
      }
    }
  }
}"#;

        let _rollout: super::Rollout = serde_json::from_str(raw_json).unwrap();
    }
}
