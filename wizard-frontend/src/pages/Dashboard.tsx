import { useState, useEffect } from "react";
import { SidebarProvider, Sidebar } from "@/components/ui/sidebar";
import { ConfigWizard } from "@/components/ConfigWizard";
import { Header } from "@/components/Header";
import { AppSidebar } from "@/components/AppSidebar";
import { HomeSection } from "@/components/HomeSection";
import { OverviewSection } from "@/components/OverviewSection";

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

const Dashboard = () => {
  const [showWizard, setShowWizard] = useState(false);
  const [wizardMode, setWizardMode] = useState<'create' | 'overview'>('create');
  const [services, setServices] = useState<Service[]>([]);
  const [activeSection, setActiveSection] = useState("home");

  const handleConfigSave = (config: Partial<Config>) => {
    // TODO: handle downloading?
    
    setShowWizard(false);
  };

  const handleWizardOpen = (mode: 'create' | 'overview') => {
    setWizardMode(mode);
    setShowWizard(true);
  };

  return (
    <SidebarProvider>
      <div className="min-h-screen w-full bg-background">
        <Header />

        <div className="flex w-full">
          <main className="flex-1">
            {activeSection === "home" && (
              <HomeSection 
                services={services}
                onWizardOpen={handleWizardOpen}
                onSectionChange={setActiveSection}
              />
            )}

            {activeSection === "overview" && (
              <OverviewSection onWizardOpen={handleWizardOpen} />
            )}

            
          </main>
        </div>

        {showWizard && (
          <ConfigWizard 
            isOpen={showWizard} 
            onClose={() => setShowWizard(false)} 
            onSave={handleConfigSave}
            existingConfigs={services.flatMap(s => s.configs)}
            mode={wizardMode}
          />
        )}
      </div>
    </SidebarProvider>
  );
};

export default Dashboard;