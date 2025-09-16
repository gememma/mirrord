import { useState, useEffect } from "react";
import { SidebarProvider, Sidebar } from "@/components/ui/sidebar";
import { ConfigWizard } from "@/components/ConfigWizard";
import { Header } from "@/components/Header";
import { AppSidebar } from "@/components/AppSidebar";
import { HomeSection } from "@/components/HomeSection";
import { OverviewSection } from "@/components/OverviewSection";
import { ConfigsSection } from "@/components/ConfigsSection";

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

  useEffect(() => {
    // Load configs from localStorage
    const savedConfigs = JSON.parse(localStorage.getItem('mirrord-configs') || '[]');
    // Group configs by service
    const groupedServices: Service[] = [];
    savedConfigs.forEach((config: Config) => {
      const existingService = groupedServices.find(s => s.name === config.service);
      if (existingService) {
        existingService.configs.push(config);
      } else {
        groupedServices.push({
          name: config.service,
          configs: [config]
        });
      }
    });
    setServices(groupedServices);
  }, []);

  const handleSetActive = (serviceIndex: number, configId: string) => {
    setServices(services.map((service, sIndex) => ({
      ...service,
      configs: service.configs.map(config => ({
        ...config,
        isActive: sIndex === serviceIndex && config.id === configId
      }))
    })));
  };

  const handleDelete = (serviceIndex: number, configId: string) => {
    const updatedServices = services.map((service, sIndex) => {
      if (sIndex === serviceIndex) {
        return {
          ...service,
          configs: service.configs.filter(config => config.id !== configId)
        };
      }
      return service;
    }).filter(service => service.configs.length > 0);

    setServices(updatedServices);
    
    // Update localStorage
    const allConfigs = updatedServices.flatMap(s => s.configs);
    localStorage.setItem('mirrord-configs', JSON.stringify(allConfigs));
  };

  const handleDuplicate = (serviceIndex: number, configId: string) => {
    const service = services[serviceIndex];
    const configToDupe = service.configs.find(c => c.id === configId);
    if (configToDupe) {
      const newConfig = {
        ...configToDupe,
        id: Date.now().toString(),
        name: `${configToDupe.name} (copy)`,
        isActive: false,
        createdAt: new Date().toISOString().split('T')[0]
      };
      
      const updatedServices = services.map((s, sIndex) => {
        if (sIndex === serviceIndex) {
          return {
            ...s,
            configs: [...s.configs, newConfig]
          };
        }
        return s;
      });
      
      setServices(updatedServices);
      
      // Update localStorage
      const allConfigs = updatedServices.flatMap(s => s.configs);
      localStorage.setItem('mirrord-configs', JSON.stringify(allConfigs));
    }
  };

  const handleConfigSave = (config: Partial<Config>) => {
    const newConfig = {
      ...config,
      id: Date.now().toString(),
      service: config.target?.split(' ')[0] || 'my-service',
      createdAt: new Date().toISOString().split('T')[0]
    };
    
    const serviceName = newConfig.service;
    const existingServiceIndex = services.findIndex(s => s.name === serviceName);
    
    let updatedServices;
    if (existingServiceIndex >= 0) {
      updatedServices = services.map((service, index) => {
        if (index === existingServiceIndex) {
          return {
            ...service,
            configs: [...service.configs, newConfig as Config]
          };
        }
        return service;
      });
    } else {
      updatedServices = [...services, {
        name: serviceName,
        configs: [newConfig as Config]
      }];
    }
    
    setServices(updatedServices);
    
    // Update localStorage
    const allConfigs = updatedServices.flatMap(s => s.configs);
    localStorage.setItem('mirrord-configs', JSON.stringify(allConfigs));
    
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
          <Sidebar>
            <AppSidebar 
              services={services}
              activeSection={activeSection}
              onSectionChange={setActiveSection}
            />
          </Sidebar>

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

            {activeSection === "configs" && (
              <ConfigsSection 
                services={services}
                onWizardOpen={handleWizardOpen}
                onSetActive={handleSetActive}
                onDelete={handleDelete}
                onDuplicate={handleDuplicate}
              />
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