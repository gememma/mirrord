use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Display, Formatter},
};

use kube::CustomResource;
use kube_target::{KubeTarget, UnknownTargetType};
pub use mirrord_config::feature::split_queues::QueueId;
use mirrord_config::{
    feature::split_queues::{SplitQueuesConfig, SqsMessageFilter},
    target::{Target, TargetConfig},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::label_selector::LabelSelector;
#[cfg(feature = "client")]
use crate::client::error::OperatorApiError;
use crate::types::LicenseInfoOwned;

pub mod kube_target;
pub mod label_selector;

pub const TARGETLESS_TARGET_NAME: &str = "targetless";

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "operator.metalbear.co",
    version = "v1",
    kind = "Target",
    root = "TargetCrd",
    namespaced
)]
pub struct TargetSpec {
    /// The kubernetes resource to target.
    pub target: KubeTarget,
}

impl TargetCrd {
    /// Creates a target name in format of `target_type.target_name.[container.container_name]`
    /// for example:
    ///
    /// - `DeploymentTarget { deployment: "nginx", container: None }` -> `deploy.nginx`;
    /// - `DeploymentTarget { deployment: "nginx", container: Some("pyrex") }` ->
    ///   `deploy.nginx.container.pyrex`;
    ///
    /// It's used to connect to a resource through the operator.
    pub fn urlfied_name(target: &Target) -> String {
        let (type_name, target, container) = match target {
            Target::Deployment(target) => ("deploy", &target.deployment, &target.container),
            Target::Pod(target) => ("pod", &target.pod, &target.container),
            Target::Rollout(target) => ("rollout", &target.rollout, &target.container),
            Target::Job(target) => ("job", &target.job, &target.container),
            Target::CronJob(target) => ("cronjob", &target.cron_job, &target.container),
            Target::StatefulSet(target) => ("statefulset", &target.stateful_set, &target.container),
            Target::Targetless => return TARGETLESS_TARGET_NAME.to_string(),
        };

        if let Some(container) = container {
            format!("{}.{}.container.{}", type_name, target, container)
        } else {
            format!("{}.{}", type_name, target)
        }
    }

    /// "targetless" ([`TARGETLESS_TARGET_NAME`]) if `None`,
    /// else <resource_type>.<resource_name>...
    pub fn target_name_by_config(target_config: &TargetConfig) -> String {
        target_config
            .path
            .as_ref()
            .map_or_else(|| TARGETLESS_TARGET_NAME.to_string(), Self::urlfied_name)
    }
}

impl TryFrom<TargetCrd> for TargetConfig {
    type Error = UnknownTargetType;

    fn try_from(crd: TargetCrd) -> Result<Self, Self::Error> {
        Ok(TargetConfig {
            path: Some(Target::try_from(crd.spec.target)?),
            namespace: crd.metadata.namespace,
        })
    }
}

pub static OPERATOR_STATUS_NAME: &str = "operator";

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "operator.metalbear.co",
    version = "v1",
    kind = "MirrordOperator",
    root = "MirrordOperatorCrd",
    status = "MirrordOperatorStatus"
)]
pub struct MirrordOperatorSpec {
    pub operator_version: String,
    pub default_namespace: String,
    /// Should be removed when we can stop supporting compatibility with versions from before the
    /// `supported_features` field was added.
    /// "Breaking" that compatibility by removing this field and then running with one old (from
    /// before the `supported_features` field) side (client or operator) would make the client
    /// think `ProxyApi` is not supported even if it is.
    #[deprecated(note = "use supported_features instead")]
    features: Option<Vec<OperatorFeatures>>,
    /// Replaces both `features` and `copy_target_enabled`. Operator versions that use a version
    /// of this code that has both this and the old fields are expected to populate this field with
    /// the full set of features they support, and the old fields with their limited info they
    /// support, for old clients.
    ///
    /// Access this info only via `supported_features()`.
    /// Optional for backwards compatibility (new clients can talk to old operators that don't send
    /// this field).
    supported_features: Option<Vec<NewOperatorFeature>>,
    pub license: LicenseInfoOwned,
    pub protocol_version: Option<String>,
    /// Should be removed when we can stop supporting compatibility with versions from before the
    /// `supported_features` field was added.
    /// "Breaking" that compatibility by removing this field and then running with one old (from
    /// before the `supported_features` field) side (client or operator) would make the client
    /// think copy target is not enabled even if it is.
    /// Optional for backwards compatibility (new clients can talk to old operators that don't send
    /// this field).
    #[deprecated(note = "use supported_features instead")]
    copy_target_enabled: Option<bool>,
}

impl MirrordOperatorSpec {
    pub fn new(
        operator_version: String,
        default_namespace: String,
        supported_features: Vec<NewOperatorFeature>,
        license: LicenseInfoOwned,
        protocol_version: Option<String>,
    ) -> Self {
        let features = supported_features
            .contains(&NewOperatorFeature::ProxyApi)
            .then(|| vec![OperatorFeatures::ProxyApi]);
        let copy_target_enabled =
            Some(supported_features.contains(&NewOperatorFeature::CopyTarget));
        #[allow(deprecated)] // deprecated objects must still be included in construction.
        Self {
            operator_version,
            default_namespace,
            supported_features: Some(supported_features),
            license,
            protocol_version,
            features,
            copy_target_enabled,
        }
    }

    /// Get a vector with the features the operator supports.
    /// Handles objects sent from old and new operators.
    // When the deprecated fields are removed, this can be changed to just return
    // `self.supported_features.unwrap_or_default()`.
    pub fn supported_features(&self) -> Vec<NewOperatorFeature> {
        self.supported_features
            .clone()
            // if supported_features was sent, just use that. If not we are dealing with an older
            // operator, so we build a vector of new features from the old fields.
            .or_else(|| {
                // object was sent by an old operator that still uses fields that are now deprecated
                #[allow(deprecated)]
                self.features.as_ref().map(|features| {
                    features
                        .iter()
                        .map(From::from)
                        .chain(
                            self.copy_target_enabled.and_then(|enabled| {
                                enabled.then_some(NewOperatorFeature::CopyTarget)
                            }),
                        )
                        .collect()
                })
            })
            // Convert `None` to empty vector since we don't expect this to often be
            // `None` (although it's ok if it is) and that way the return type is simpler.
            .unwrap_or_default()
    }

    #[cfg(feature = "client")]
    pub fn require_feature(&self, feature: NewOperatorFeature) -> Result<(), OperatorApiError> {
        if self.supported_features().contains(&feature) {
            Ok(())
        } else {
            Err(OperatorApiError::UnsupportedFeature {
                feature,
                operator_version: self.operator_version.clone(),
            })
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct MirrordOperatorStatus {
    pub sessions: Vec<Session>,
    pub statistics: Option<MirrordOperatorStatusStatistics>,

    /// Option because added later.
    /// (copy-target pod name, copy-target resource)
    pub copy_targets: Option<Vec<(String, CopyTargetCrd)>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct MirrordOperatorStatusStatistics {
    pub dau: usize,
    pub mau: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Session {
    pub id: Option<String>,
    pub duration_secs: u64,
    pub user: String,
    pub target: String,
    pub namespace: Option<String>,
    pub locked_ports: Option<Vec<(u16, String, Option<String>)>>,
}

/// Resource used to access the operator's session management routes.
///
/// - `kind = Session` controls how [`kube`] generates the route, in this case it becomes
///   `/sessions`;
/// - `root = "SessionCrd"` is the json return value we get from this resource's API;
/// - `SessionSpec` itself contains the custom data we want to pass in the the response, which in
///   this case is nothing;
///
/// The [`SessionCrd`] is used to provide the k8s_openapi `APIResource`, see `API_RESOURCE_LIST` in
/// the operator.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "operator.metalbear.co",
    version = "v1",
    kind = "Session",
    root = "SessionCrd"
)]
pub struct SessionSpec;

/// Since this enum does not have a variant marked with `#[serde(other)]`, and is present like that
/// in released clients, existing clients would fail to parse any new variant. This means the
/// operator can never send anything but the one existing variant, otherwise the client will error
/// out.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub enum OperatorFeatures {
    ProxyApi,
    // DON'T ADD VARIANTS - old clients won't be able to deserialize them.
    // Add new features in NewOperatorFeature
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub enum NewOperatorFeature {
    ProxyApi,
    CopyTarget,
    SqsQueueSplitting,
    SessionManagement,
    /// This variant is what a client sees when the operator includes a feature the client is not
    /// yet aware of, because it was introduced in a version newer than the client's.
    #[schemars(skip)]
    #[serde(other, skip_serializing)]
    Unknown,
}

impl Display for NewOperatorFeature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            NewOperatorFeature::ProxyApi => "proxy API",
            NewOperatorFeature::CopyTarget => "copy target",
            NewOperatorFeature::SqsQueueSplitting => "SQS queue splitting",
            NewOperatorFeature::Unknown => "unknown feature",
            NewOperatorFeature::SessionManagement => "session management",
        };
        f.write_str(name)
    }
}

impl From<&OperatorFeatures> for NewOperatorFeature {
    fn from(old_feature: &OperatorFeatures) -> Self {
        match old_feature {
            OperatorFeatures::ProxyApi => NewOperatorFeature::ProxyApi,
        }
    }
}

/// This [`Resource`](kube::Resource) represents a copy pod created from an existing [`Target`]
/// (operator's copy pod feature).
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "operator.metalbear.co",
    version = "v1",
    kind = "CopyTarget",
    root = "CopyTargetCrd",
    namespaced
)]
pub struct CopyTargetSpec {
    /// Original target. Only [`Target::Pod`] and [`Target::Deployment`] are accepted.
    pub target: Target,
    /// How long should the operator keep this pod alive after its creation.
    /// The pod is deleted when this timout has expired and there are no connected clients.
    pub idle_ttl: Option<u32>,
    /// Should the operator scale down target deployment to 0 while this pod is alive.
    /// Ignored if [`Target`] is not [`Target::Deployment`].
    pub scale_down: bool,
    /// Split queues client side configuration.
    pub split_queues: Option<SplitQueuesConfig>,
}

/// Features and operations that can be blocked by a `MirrordPolicy`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")] // StealWithoutFilter -> steal-without-filter in yaml.
pub enum BlockedFeature {
    /// Blocks stealing traffic in any way (without or without filter).
    Steal,
    /// Blocks stealing traffic without specifying (any) filter. Client can still specify a
    /// filter that matches anything.
    StealWithoutFilter,
}

/// Custom resource for policies that limit what mirrord features users can use.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    // The operator group is handled by the operator, we want policies to be handled by k8s.
    group = "policies.mirrord.metalbear.co",
    version = "v1alpha",
    kind = "MirrordPolicy",
    namespaced
)]
#[serde(rename_all = "camelCase")] // target_path -> targetPath in yaml.
pub struct MirrordPolicySpec {
    /// Specify the targets for which this policy applies, in the pod/my-pod deploy/my-deploy
    /// notation. Targets can be matched using `*` and `?` where `?` matches exactly one
    /// occurrence of any character and `*` matches arbitrary many (including zero) occurrences
    /// of any character. If not specified, this policy does not depend on the target's path.
    pub target_path: Option<String>,

    /// If specified in a policy, the policy will only apply to targets with labels that match all
    /// of the selector's rules.
    pub selector: Option<LabelSelector>,

    // TODO: make the k8s list type be set/map to prevent duplicates.
    /// List of features and operations blocked by this policy.
    pub block: Vec<BlockedFeature>,
}

/// Set where the application reads the name of the queue from, so that mirrord can find that queue,
/// split it, and temporarily change the name there to the name of the branch queue when splitting.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")] // EnvVar -> envVar in yaml.
pub enum QueueNameSource {
    EnvVar(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")] // name_source -> nameSource in yaml.
pub struct SqsQueueDetails {
    /// Where the application gets the queue name from. Will be used to read messages from that
    /// queue and distribute them to the output queues. When running with mirrord and splitting
    /// this queue, applications will get a modified name from that source.
    pub name_source: QueueNameSource,

    /// These tags will be set for all temporary SQS queues created by mirrord for queues defined
    /// in this MirrordWorkloadQueueRegistry, alongside with the original tags of the respective
    /// original queue. In case of a collision, the temporary queue will get the value from the
    /// tag passed in here.
    pub tags: Option<HashMap<String, String>>,
}

/// The details of a queue that should be split.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(tag = "queueType")]
pub enum SplitQueue {
    /// Amazon SQS
    #[serde(rename = "SQS")]
    Sqs(SqsQueueDetails),
}

/// A workload that is a consumer of a queue that is being split.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub struct QueueConsumer {
    pub name: String,
    /// If a container is not specified, the workload queue registry will apply to every run that
    /// targets any of the workload's containers.
    pub container: Option<String>,
    pub workload_type: QueueConsumerType,
}

/// A workload that is a consumer of a queue that is being split.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
pub enum QueueConsumerType {
    Deployment,

    Rollout,

    #[schemars(skip)]
    #[serde(other, skip_serializing)]
    Unsupported,
}

impl Display for QueueConsumerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QueueConsumerType::Deployment => write!(f, "deployment"),
            QueueConsumerType::Rollout => write!(f, "rollout"),
            QueueConsumerType::Unsupported => write!(f, "unsupported"),
        }
    }
}

impl QueueConsumer {
    /// For self that is the queue consumer of a run, test if a given registry object is the correct
    /// registry for this run.
    pub fn registry_matches(&self, registry: &MirrordWorkloadQueueRegistry) -> bool {
        let registry_consumer = &registry.spec.consumer;
        self.workload_type == registry_consumer.workload_type
            && self.name == registry_consumer.name
            && (self.container == registry_consumer.container
            // If registry does not specify a container, it applies to all runs with
            // this target, regardless of what container they are targeting.
            || registry_consumer.container.is_none())
    }
}

impl Display for QueueConsumer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ref container) = self.container {
            write!(
                f,
                "{}/{}/container/{container}",
                self.workload_type, self.name
            )
        } else {
            write!(f, "{}/{}", self.workload_type, self.name)
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct QueueNameUpdate {
    pub original_name: String,
    pub output_name: String,
}

/// Details retrieved from K8s resources once the splitter is active, used on filter session
/// creation to determine the required config changes that make the application use the
/// output queues instead of the original.
// This status struct is not optimal in that it contains redundant information. This makes the
// controller's code a bit simpler.
// Some information is present in the spec, but it is organized differently.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct ActiveSqsSplits {
    /// For each queue_id, the actual queue name as retrieved from the target's pod spec or config
    /// map, together with the name of its temporary output queue.
    pub queue_names: BTreeMap<QueueId, QueueNameUpdate>,

    /// Names of env vars that contain the queue name directly in the pod template, without config
    /// map refs, mapped to their queue id.
    pub direct_env_vars: HashMap<String, QueueId>,

    pub env_updates: BTreeMap<String, QueueNameUpdate>,
}

impl ActiveSqsSplits {
    pub fn output_queue_names(&self) -> Vec<&str> {
        self.queue_names
            .values()
            .map(|QueueNameUpdate { output_name, .. }| output_name.as_str())
            .collect()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")] // active_filters -> activeFilters
pub struct WorkloadQueueRegistryStatus {
    /// Optional even though it's currently the only field, because in the future there will be
    /// fields for other queue types.
    pub sqs_details: Option<ActiveSqsSplits>,
}

/// Defines a Custom Resource that holds a central configuration for splitting queues for a
/// QueueConsumer (a target workload for which queues should be split).
///
/// This means there should be 1 such resource per queue splitting target.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "queues.mirrord.metalbear.co",
    version = "v1alpha",
    kind = "MirrordWorkloadQueueRegistry",
    shortname = "qs",
    status = "WorkloadQueueRegistryStatus",
    namespaced
)]
pub struct MirrordWorkloadQueueRegistrySpec {
    /// A map of the queues that should be split.
    /// The key is used by users to associate filters to the right queues.
    pub queues: BTreeMap<QueueId, SplitQueue>,

    /// The resource (deployment or Argo rollout) that reads from the queues.
    pub consumer: QueueConsumer,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename = "SQSSplitDetails", rename_all = "camelCase")]
pub struct SqsSplitDetails {
    /// Queue ID -> old and new queue names.
    pub queue_names: BTreeMap<QueueId, QueueNameUpdate>,

    // A bit redundant, because the registry resource status has the mapping from env var name
    // to queue id, and `queue_names` has the mapping from queue id to name update, but, saving
    // it here in the form that is useful to reader, for simplicity and readability.
    /// Env var name -> old and new queue names.
    pub env_updates: BTreeMap<String, QueueNameUpdate>,
}

/// Representation of Sqs errors for the status of SQS session resources.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SqsSessionError {
    /// HTTP code for operator response.
    pub status_code: u16,

    /// Human-readable explanation of what went wrong.
    pub reason: String,
}

impl Display for SqsSessionError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}", self.reason)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename = "SQSSessionStatus")]
pub enum SqsSessionStatus {
    // kube-rs does not allow mixing unit variants with tuple/struct variants, so this variant
    // has to be a tuple/struct too. If we leave the tuple empty, k8s complains about an object
    // without any items, and kube-rs does not support internally tagged enums, so we actually
    // have to put something in there, even if we don't actually care about that info.
    Starting {
        start_time_utc: String,
    },
    /// SQS operator sets this status before it starts registering filters, so that if anything
    /// fails during the registration of filters, we have all the queues we need to delete on
    /// cleanup.
    RegisteringFilters(SqsSplitDetails),
    Ready(SqsSplitDetails),
    StartError(SqsSessionError),
    CleanupError {
        error: SqsSessionError,
        details: Option<SqsSplitDetails>,
    },
}

impl SqsSessionStatus {
    pub fn get_split_details(&self) -> Option<&SqsSplitDetails> {
        match self {
            SqsSessionStatus::RegisteringFilters(details) | SqsSessionStatus::Ready(details) => {
                Some(details)
            }
            SqsSessionStatus::CleanupError { details, .. } => details.as_ref(),
            _ => None,
        }
    }
}

/// The [`kube::runtime::wait::Condition`] trait is auto-implemented for this function.
/// To be used in [`kube::runtime::wait::await_condition`].
pub fn is_session_ready(session: Option<&MirrordSqsSession>) -> bool {
    session
        .and_then(|session| session.status.as_ref())
        .map(|status| {
            matches!(
                status,
                SqsSessionStatus::Ready(..)
                    | SqsSessionStatus::StartError(..)
                    | SqsSessionStatus::CleanupError { .. }
            )
        })
        .unwrap_or_default()
}

/// The operator creates this object when a user runs mirrord against a target that is a queue
/// consumer.
#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
group = "queues.mirrord.metalbear.co",
version = "v1alpha",
kind = "MirrordSQSSession",
root = "MirrordSqsSession", // for Rust naming conventions (Sqs, not SQS)
status = "SqsSessionStatus",
namespaced
)]
#[serde(rename_all = "camelCase")] // queue_filters -> queueFilters
pub struct MirrordSqsSessionSpec {
    /// For each queue_id, a mapping from attribute name, to attribute value regex.
    /// The queue_id for a queue is determined at the queue registry. It is not (necessarily)
    /// The name of the queue on AWS.
    pub queue_filters: HashMap<QueueId, SqsMessageFilter>,

    /// The target of this session.
    pub queue_consumer: QueueConsumer,

    /// The id of the mirrord exec session, from the operator.
    // The Kubernetes API can't deal with 64 bit numbers (with most significant bit set)
    // so we save that field as a (HEX) string even though its source is a u64
    pub session_id: String,
}
