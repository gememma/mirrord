import { Home, FileCode2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { SidebarContent, SidebarGroup, SidebarGroupContent, SidebarGroupLabel, SidebarMenu, SidebarMenuItem, SidebarMenuButton } from "@/components/ui/sidebar";

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

interface AppSidebarProps {
  services: Service[];
  activeSection: string;
  onSectionChange: (section: string) => void;
}

export const AppSidebar = ({ services, activeSection, onSectionChange }: AppSidebarProps) => {
  return (
    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>Dashboard</SidebarGroupLabel>
        <SidebarGroupContent>
          <SidebarMenu>
            <SidebarMenuItem>
              <SidebarMenuButton isActive={activeSection === "home"} onClick={() => onSectionChange("home")}>
                <Home className="h-4 w-4" />
                <span>Home</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
            <SidebarMenuItem>
              <SidebarMenuButton isActive={activeSection === "configs"} onClick={() => onSectionChange("configs")}>
                <FileCode2 className="h-4 w-4" />
                <span>Config Files</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarGroupContent>
      </SidebarGroup>
      
      {services.length > 0 && (
        <SidebarGroup>
          <SidebarGroupLabel>Saved Configurations</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {services.flatMap(service => 
                service.configs.map(config => (
                  <SidebarMenuItem key={config.id}>
                    <SidebarMenuButton
                      onClick={() => onSectionChange("configs")}
                      className="flex items-center justify-between w-full"
                    >
                      <div className="flex flex-col items-start min-w-0 flex-1">
                        <span className="text-sm font-medium truncate">{config.name}</span>
                        <span className="text-xs text-muted-foreground truncate">{service.name}</span>
                      </div>
                      {config.isActive && (
                        <Badge className="bg-primary text-primary-foreground text-xs ml-2">
                          Active
                        </Badge>
                      )}
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))
              )}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      )}
    </SidebarContent>
  );
};
