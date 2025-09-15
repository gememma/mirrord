# API and Integration Documentation

This document provides comprehensive documentation for the mirrord Configuration Wizard Frontend API, data structures, and integration points.

## 📋 Table of Contents

- [Data Types](#data-types)
- [Configuration API](#configuration-api)
- [Storage API](#storage-api)
- [Component APIs](#component-apis)
- [Integration Points](#integration-points)
- [Mock Data](#mock-data)
- [Error Handling](#error-handling)

## 🏗️ Data Types

### Core Configuration Types

#### ConfigData
The main configuration interface that represents a complete mirrord configuration.

```typescript
interface ConfigData {
  name: string;                    // Configuration name
  target: string;                  // Kubernetes target (namespace/kind/name)
  targetType: string;              // Resource type (deployment, statefulset, etc.)
  namespace: string;               // Kubernetes namespace
  service?: string;                // Service name (derived from target)
  fileSystem: FileSystemConfig;    // File system configuration
  network: NetworkConfig;          // Network configuration
  environment: EnvironmentConfig;  // Environment variables configuration
  agent: AgentConfig;              // Agent-specific settings
  isActive: boolean;               // Whether this config is currently active
}
```

#### FileSystemConfig
File system access configuration.

```typescript
interface FileSystemConfig {
  enabled: boolean;                // Whether file system access is enabled
  mode: "read" | "write" | "local"; // Access mode
  rules: Array<{                   // File system access rules
    mode: "read" | "write" | "local";
    filter: string;                // Path filter pattern
  }>;
}
```

#### NetworkConfig
Network traffic configuration.

```typescript
interface NetworkConfig {
  incoming: IncomingConfig;        // Incoming traffic configuration
  outgoing: OutgoingConfig;        // Outgoing traffic configuration
  dns: DnsConfig;                  // DNS configuration
}

interface IncomingConfig {
  enabled: boolean;                // Whether incoming traffic is enabled
  mode: "steal" | "mirror";       // Traffic handling mode
  httpFilter: Array<{              // HTTP traffic filters
    type: "header" | "method" | "content" | "path";
    value: string;                 // Filter value
    matchType?: "exact" | "regex"; // Match type for headers
  }>;
  filterOperator: "AND" | "OR";    // Logic operator for multiple filters
  ports: Array<{                   // Port mappings
    remote: string;                // Remote port
    local: string;                 // Local port
  }>;
}

interface OutgoingConfig {
  enabled: boolean;                // Whether outgoing traffic is enabled
  protocol: "tcp" | "udp" | "both"; // Protocol filter
  filter: string;                  // Traffic filter
  filterTarget: "remote" | "local"; // Filter target
}

interface DnsConfig {
  enabled: boolean;                // Whether DNS is enabled
  filter: string;                  // DNS filter pattern
}
```

#### EnvironmentConfig
Environment variables configuration.

```typescript
interface EnvironmentConfig {
  enabled: boolean;                // Whether environment is enabled
  include: string;                 // Include pattern
  exclude: string;                 // Exclude pattern
  override: string;                // Override values
}
```

#### AgentConfig
Agent-specific configuration.

```typescript
interface AgentConfig {
  scaledown: boolean;              // Whether to scale down target
  copyTarget: boolean;             // Whether to copy target configuration
}
```

### Service and UI Types

#### Service
Represents a group of configurations for a service.

```typescript
interface Service {
  name: string;                    // Service name
  configs: Config[];               // Array of configurations
}

interface Config extends ConfigData {
  id: string;                      // Unique configuration ID
  createdAt: string;               // Creation date
  service: string;                 // Service name
}
```

#### FeatureConfig
Simplified configuration for mirrord feature flags.

```typescript
type FeatureConfig = {
  network?: {
    incoming?: {
      mode: "steal" | "mirror";
      http_filter?: Record<string, Array<Record<string, string>>>;
      ports?: Array<Record<string, string>>;
    };
    outgoing?: {
      filter: Record<string, string | Record<string, string>>;
    };
  };
  fs?: {
    mode: "read" | "write" | "local";
  };
  env?: {
    include?: string;
    exclude?: string;
    override?: string;
  };
};
```

## 🔧 Configuration API

### Configuration Generation

#### generateConfigJson()
Generates a mirrord-compatible JSON configuration from the internal ConfigData format.

```typescript
function generateConfigJson(config: ConfigData): string
```

**Parameters:**
- `config: ConfigData` - The configuration to convert

**Returns:**
- `string` - JSON string representation of the configuration

**Example:**
```typescript
const config: ConfigData = {
  name: "my-config",
  target: "default/deployment/api-service",
  targetType: "deployment",
  namespace: "default",
  // ... other properties
};

const jsonConfig = generateConfigJson(config);
console.log(jsonConfig);
// Output: {"target": "default/deployment/api-service", "agent": {...}, "feature": {...}}
```

### Configuration Validation

#### validateJson()
Validates JSON configuration syntax.

```typescript
function validateJson(jsonString: string): boolean
```

**Parameters:**
- `jsonString: string` - JSON string to validate

**Returns:**
- `boolean` - True if valid JSON, false otherwise

**Example:**
```typescript
const isValid = validateJson('{"target": "test"}');
if (isValid) {
  console.log("Valid JSON");
} else {
  console.log("Invalid JSON");
}
```

#### updateConfigFromJson()
Updates configuration state from JSON input.

```typescript
function updateConfigFromJson(jsonString: string): void
```

**Parameters:**
- `jsonString: string` - JSON configuration string

**Example:**
```typescript
const jsonConfig = '{"target": "default/deployment/api", "agent": {"scaledown": true}}';
updateConfigFromJson(jsonConfig);
```

## 💾 Storage API

### Local Storage Operations

The application uses localStorage for persistence. All storage operations are centralized in the configuration management functions.

#### Storage Keys
- `mirrord-configs`: Array of saved configurations
- `mirrord-onboarding-completed`: Boolean flag for onboarding completion

#### Configuration Storage

```typescript
// Save configuration
function saveConfiguration(config: ConfigData): void {
  const existingConfigs = JSON.parse(localStorage.getItem('mirrord-configs') || '[]');
  const newConfig = {
    ...config,
    id: Date.now().toString(),
    service: config.target?.split(' ')[0] || 'my-service',
    createdAt: new Date().toISOString().split('T')[0]
  };
  
  const updatedConfigs = [...existingConfigs, newConfig];
  localStorage.setItem('mirrord-configs', JSON.stringify(updatedConfigs));
}

// Load configurations
function loadConfigurations(): Config[] {
  return JSON.parse(localStorage.getItem('mirrord-configs') || '[]');
}

// Delete configuration
function deleteConfiguration(configId: string): void {
  const configs = loadConfigurations();
  const updatedConfigs = configs.filter(config => config.id !== configId);
  localStorage.setItem('mirrord-configs', JSON.stringify(updatedConfigs));
}

// Update configuration
function updateConfiguration(configId: string, updates: Partial<ConfigData>): void {
  const configs = loadConfigurations();
  const updatedConfigs = configs.map(config => 
    config.id === configId ? { ...config, ...updates } : config
  );
  localStorage.setItem('mirrord-configs', JSON.stringify(updatedConfigs));
}
```

#### Onboarding State

```typescript
// Mark onboarding as completed
function completeOnboarding(): void {
  localStorage.setItem('mirrord-onboarding-completed', 'true');
}

// Check if onboarding is completed
function isOnboardingCompleted(): boolean {
  return localStorage.getItem('mirrord-onboarding-completed') === 'true';
}
```

## 🧩 Component APIs

### ConfigWizard API

#### Props Interface
```typescript
interface ConfigWizardProps {
  isOpen: boolean;                    // Controls wizard visibility
  onClose: () => void;               // Close callback
  onSave: (config: ConfigData) => void; // Save callback
  existingConfigs?: ConfigData[];    // Existing configurations
  mode?: 'create' | 'overview';      // Wizard mode
}
```

#### Usage Example
```typescript
import { ConfigWizard } from '@/components/ConfigWizard';

function MyComponent() {
  const [showWizard, setShowWizard] = useState(false);
  const [configs, setConfigs] = useState<ConfigData[]>([]);

  const handleSave = (config: ConfigData) => {
    setConfigs(prev => [...prev, config]);
    setShowWizard(false);
  };

  return (
    <ConfigWizard
      isOpen={showWizard}
      onClose={() => setShowWizard(false)}
      onSave={handleSave}
      existingConfigs={configs}
      mode="create"
    />
  );
}
```

### Dashboard API

#### State Management
```typescript
interface DashboardState {
  services: Service[];               // Grouped configurations
  activeSection: string;            // Current section
  showWizard: boolean;              // Wizard visibility
  wizardMode: 'create' | 'overview'; // Wizard mode
}
```

#### Action Handlers
```typescript
interface DashboardActions {
  handleSetActive: (serviceIndex: number, configId: string) => void;
  handleDelete: (serviceIndex: number, configId: string) => void;
  handleDuplicate: (serviceIndex: number, configId: string) => void;
  handleConfigSave: (config: Partial<Config>) => void;
  handleWizardOpen: (mode: 'create' | 'overview') => void;
}
```

### AppSidebar API

#### Props Interface
```typescript
interface AppSidebarProps {
  services: Service[];                    // Services with configurations
  activeSection: string;                 // Currently active section
  onSectionChange: (section: string) => void; // Section change handler
}
```

## 🔌 Integration Points

### React Router Integration

The application uses React Router for navigation between pages.

```typescript
// Route configuration
const routes = [
  { path: "/", element: <Index /> },
  { path: "/dashboard", element: <Dashboard /> },
  { path: "/style-guide", element: <StyleGuide /> },
  { path: "*", element: <NotFound /> }
];

// Navigation hooks
import { useNavigate } from 'react-router-dom';

function MyComponent() {
  const navigate = useNavigate();
  
  const handleNavigation = () => {
    navigate('/dashboard');
  };
}
```

### React Query Integration

The application uses TanStack Query for state management and caching.

```typescript
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const queryClient = new QueryClient();

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      {/* App components */}
    </QueryClientProvider>
  );
}
```

### Toast Notifications

Integration with toast notification system.

```typescript
import { useToast } from '@/hooks/use-toast';

function MyComponent() {
  const { toast } = useToast();
  
  const handleAction = () => {
    toast({
      title: "Success",
      description: "Configuration saved successfully",
    });
  };
}
```

## 🎭 Mock Data

### Kubernetes Resources

The application includes mock data for Kubernetes resources to simulate target selection.

```typescript
const mockTargets = [
  {
    name: "api-service",
    namespace: "default",
    kind: "deployment"
  },
  {
    name: "frontend-app",
    namespace: "default",
    kind: "deployment"
  },
  {
    name: "database",
    namespace: "production",
    kind: "statefulset"
  },
  // ... more targets
];

const mockNamespaces = [
  "default",
  "production",
  "system",
  "kube-system",
  "monitoring"
];
```

### Configuration Templates

Pre-defined configuration templates for different use cases.

```typescript
const boilerplateConfigs = [
  {
    id: "steal",
    title: "Filtering mode",
    description: "Suitable for scenarios where you want to see how your changes impact remote environment while reducing the impact radius",
    features: ["steal mode", "selective traffic"],
    icon: Filter,
    color: "text-purple-500"
  },
  {
    id: "mirror",
    title: "Mirror mode",
    description: "This is useful when you want the remote target to serve requests and you're okay with one request being handled twice",
    features: ["mirror mode"],
    icon: Copy,
    color: "text-blue-500"
  },
  {
    id: "replace",
    title: "Replace mode",
    description: "Suitable for scenarios where you have your own namespace/cluster and you're okay with replacing the remote service entirely",
    features: ["steal mode", "copy target", "scale down"],
    icon: Repeat,
    color: "text-orange-500"
  }
];
```

## ⚠️ Error Handling

### Configuration Validation Errors

```typescript
interface ValidationError {
  field: string;
  message: string;
  type: 'required' | 'invalid' | 'format';
}

function validateConfiguration(config: ConfigData): ValidationError[] {
  const errors: ValidationError[] = [];
  
  if (!config.name) {
    errors.push({
      field: 'name',
      message: 'Configuration name is required',
      type: 'required'
    });
  }
  
  if (!config.target) {
    errors.push({
      field: 'target',
      message: 'Target is required',
      type: 'required'
    });
  }
  
  return errors;
}
```

### JSON Parsing Errors

```typescript
function safeJsonParse(jsonString: string): { success: boolean; data?: any; error?: string } {
  try {
    const data = JSON.parse(jsonString);
    return { success: true, data };
  } catch (error) {
    return { 
      success: false, 
      error: error instanceof Error ? error.message : 'Unknown error' 
    };
  }
}
```

### Storage Errors

```typescript
function safeStorageOperation<T>(operation: () => T, fallback: T): T {
  try {
    return operation();
  } catch (error) {
    console.error('Storage operation failed:', error);
    return fallback;
  }
}

// Usage
const configs = safeStorageOperation(
  () => JSON.parse(localStorage.getItem('mirrord-configs') || '[]'),
  []
);
```

## 🔄 State Management Patterns

### Configuration State

```typescript
interface ConfigurationState {
  configs: Config[];
  services: Service[];
  activeConfig: Config | null;
  loading: boolean;
  error: string | null;
}

const useConfigurationState = () => {
  const [state, setState] = useState<ConfigurationState>({
    configs: [],
    services: [],
    activeConfig: null,
    loading: false,
    error: null
  });

  const loadConfigurations = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      const configs = JSON.parse(localStorage.getItem('mirrord-configs') || '[]');
      const services = groupConfigurationsByService(configs);
      setState(prev => ({ 
        ...prev, 
        configs, 
        services, 
        loading: false 
      }));
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: 'Failed to load configurations', 
        loading: false 
      }));
    }
  }, []);

  return { ...state, loadConfigurations };
};
```

### Wizard State

```typescript
interface WizardState {
  currentStep: string;
  config: ConfigData;
  validationErrors: ValidationError[];
  isDirty: boolean;
}

const useWizardState = (initialConfig: ConfigData) => {
  const [state, setState] = useState<WizardState>({
    currentStep: 'target',
    config: initialConfig,
    validationErrors: [],
    isDirty: false
  });

  const updateConfig = useCallback((updates: Partial<ConfigData>) => {
    setState(prev => ({
      ...prev,
      config: { ...prev.config, ...updates },
      isDirty: true
    }));
  }, []);

  const validateStep = useCallback((step: string) => {
    const errors = validateConfiguration(state.config);
    setState(prev => ({ ...prev, validationErrors: errors }));
    return errors.length === 0;
  }, [state.config]);

  return { ...state, updateConfig, validateStep };
};
```

## 📊 Performance Considerations

### Memoization

```typescript
// Memoize expensive calculations
const groupedServices = useMemo(() => 
  groupConfigurationsByService(configs), 
  [configs]
);

// Memoize component props
const memoizedProps = useMemo(() => ({
  services: groupedServices,
  onConfigSelect: handleConfigSelect
}), [groupedServices, handleConfigSelect]);
```

### Lazy Loading

```typescript
// Lazy load heavy components
const ConfigWizard = lazy(() => import('@/components/ConfigWizard'));

// Use Suspense for loading states
<Suspense fallback={<div>Loading...</div>}>
  <ConfigWizard />
</Suspense>
```

### Debouncing

```typescript
// Debounce search input
const debouncedSearch = useMemo(
  () => debounce((query: string) => {
    setSearchResults(searchTargets(query));
  }, 300),
  []
);
```

This API documentation provides comprehensive coverage of all interfaces, data structures, and integration points in the mirrord Configuration Wizard Frontend.
