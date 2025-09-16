import { Plus, Save, Edit3, Copy, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface Config {
  id: string;
  name: string;
  target: string;
  targetType: string;
  service: string;
  namespace: string;
  isActive: boolean;
  createdAt: string;
  fileSystem: {
    enabled: boolean;
    mode: "read" | "write" | "local";
    rules: Array<{
      mode: "read" | "write" | "local";
      filter: string;
    }>;
  };
  network: {
    incoming: {
      enabled: boolean;
      mode: "steal" | "mirror";
      httpFilter: Array<{
        type: "header" | "method" | "content" | "path";
        value: string;
      }>;
      filterOperator: "AND" | "OR";
      ports: Array<{
        remote: string;
        local: string;
      }>;
    };
    outgoing: {
      enabled: boolean;
      protocol: "tcp" | "udp" | "both";
      filter: string;
      filterTarget: "remote" | "local";
    };
    dns: {
      enabled: boolean;
      filter: string;
    };
  };
  environment: {
    enabled: boolean;
    include: string;
    exclude: string;
    override: string;
  };
  agent: {
    scaledown: boolean;
    copyTarget: boolean;
  };
}

interface Service {
  name: string;
  configs: Config[];
}

interface ConfigsSectionProps {
  services: Service[];
  onWizardOpen: (mode: 'create' | 'overview') => void;
  onSetActive: (serviceIndex: number, configId: string) => void;
  onDelete: (serviceIndex: number, configId: string) => void;
  onDuplicate: (serviceIndex: number, configId: string) => void;
}

export const ConfigsSection = ({ 
  services, 
  onWizardOpen, 
  onSetActive, 
  onDelete, 
  onDuplicate 
}: ConfigsSectionProps) => {
  return (
    <div className="p-6">
      <div className="max-w-6xl">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold mb-2">Configuration Files</h1>
            <p className="text-muted-foreground">
              Manage your mirrord.json configurations
            </p>
          </div>
          <Button onClick={() => onWizardOpen('create')} className="bg-gradient-primary hover:shadow-glow transition-all duration-300">
            <Plus className="h-4 w-4 mr-2" />
            New Config
          </Button>
        </div>

        {services.length === 0 ? (
          <Card className="bg-gradient-card border-border/50 shadow-glow">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Plus className="h-5 w-5" />
                Create Configuration
              </CardTitle>
              <CardDescription>
                Use our wizard to create a working mirrord.json config without manual editing
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button onClick={() => onWizardOpen('create')} className="bg-gradient-primary">
                <Plus className="h-4 w-4 mr-2" />
                Create Your First Config
              </Button>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-6">
            {services.map((service, serviceIndex) => (
              <div key={service.name} className="space-y-4">
                <div className="flex items-center gap-3">
                  <h2 className="text-xl font-semibold">{service.name}</h2>
                  <Badge variant="secondary" className="text-xs">
                    {service.configs.length} config{service.configs.length !== 1 ? 's' : ''}
                  </Badge>
                </div>
                
                <div className="grid gap-3 ml-4">
                  {service.configs.map(config => (
                    <Card key={config.id} className={`transition-all duration-300 ${config.isActive ? 'bg-gradient-card border-primary/50 shadow-glow' : 'bg-card hover:bg-gradient-card/50'}`}>
                      <CardContent className="p-4">
                        <div className="flex items-center justify-between">
                          <div className="flex-1">
                            <div className="flex items-center gap-3 mb-2">
                              <h3 className="text-lg font-medium">{config.name}</h3>
                              {config.isActive && (
                                <Badge className="bg-primary text-primary-foreground text-xs">
                                  ACTIVE
                                </Badge>
                              )}
                            </div>
                            <p className="text-muted-foreground text-sm mb-3">
                              Target: {config.target}
                            </p>
                            <div className="flex items-center gap-4 text-xs text-muted-foreground">
                              <span>Created: {config.createdAt}</span>
                              <div className="flex items-center gap-1">
                                {config.fileSystem.enabled && <Badge variant="outline" className="text-xs">FS</Badge>}
                                {(config.network.incoming.enabled || config.network.outgoing.enabled || config.network.dns.enabled) && <Badge variant="outline" className="text-xs">NET</Badge>}
                                {config.environment.enabled && <Badge variant="outline" className="text-xs">ENV</Badge>}
                              </div>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            {!config.isActive && (
                              <Button variant="outline" size="sm" onClick={() => onSetActive(serviceIndex, config.id)}>
                                <Save className="h-4 w-4 mr-2" />
                                Set Active
                              </Button>
                            )}
                            <Button variant="outline" size="sm">
                              <Edit3 className="h-4 w-4" />
                            </Button>
                            <Button variant="outline" size="sm" onClick={() => onDuplicate(serviceIndex, config.id)}>
                              <Copy className="h-4 w-4" />
                            </Button>
                            <Button variant="outline" size="sm" onClick={() => onDelete(serviceIndex, config.id)} className="text-destructive hover:text-destructive-foreground hover:bg-destructive">
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
