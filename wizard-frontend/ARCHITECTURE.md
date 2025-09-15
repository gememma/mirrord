# Architecture and Design Documentation

This document provides a comprehensive overview of the mirrord Configuration Wizard Frontend architecture, design decisions, and system organization.

## 📋 Table of Contents

- [System Overview](#system-overview)
- [Architecture Patterns](#architecture-patterns)
- [Component Architecture](#component-architecture)
- [State Management](#state-management)
- [Data Flow](#data-flow)
- [Design System](#design-system)
- [Performance Architecture](#performance-architecture)
- [Security Considerations](#security-considerations)
- [Scalability](#scalability)

## 🏗️ System Overview

### High-Level Architecture

The mirrord Configuration Wizard Frontend is a single-page application (SPA) built with React and TypeScript. It follows a modern, component-based architecture with clear separation of concerns.

```
┌─────────────────────────────────────────────────────────────┐
│                    Browser Environment                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   React App     │  │   React Router  │  │  LocalStorage│ │
│  │   (SPA)         │  │   (Navigation)  │  │  (Persistence)│ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Vite Dev      │  │   TypeScript    │  │   Tailwind   │ │
│  │   Server        │  │   Compiler      │  │   CSS        │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Technologies

- **Frontend Framework**: React 18 with TypeScript
- **Build Tool**: Vite for fast development and optimized builds
- **Styling**: Tailwind CSS with custom design system
- **UI Components**: shadcn/ui (Radix UI primitives)
- **State Management**: React hooks with localStorage persistence
- **Routing**: React Router DOM for client-side navigation
- **Form Handling**: React Hook Form with validation
- **Icons**: Lucide React icon library

## 🎯 Architecture Patterns

### Component-Based Architecture

The application follows a hierarchical component structure with clear responsibilities:

```
App
├── Router
│   ├── Index (Landing/Onboarding)
│   ├── Dashboard
│   └── StyleGuide
└── Providers
    ├── QueryClientProvider
    ├── TooltipProvider
    └── ToasterProvider
```

### Separation of Concerns

#### 1. **Presentation Layer**
- React components for UI rendering
- Event handling and user interactions
- Visual feedback and animations

#### 2. **Business Logic Layer**
- Custom hooks for state management
- Configuration generation and validation
- Data transformation utilities

#### 3. **Data Layer**
- localStorage for persistence
- Mock data for development
- Configuration data structures

#### 4. **Service Layer**
- API integration points (future)
- External service communication
- Data fetching and caching

### Design Patterns

#### 1. **Container/Presentational Pattern**
```typescript
// Container Component (Logic)
const ConfigWizardContainer = () => {
  const [config, setConfig] = useState(initialConfig);
  const [isValid, setIsValid] = useState(false);
  
  const handleSave = useCallback((newConfig) => {
    // Business logic
    setConfig(newConfig);
    setIsValid(validateConfig(newConfig));
  }, []);
  
  return (
    <ConfigWizard
      config={config}
      isValid={isValid}
      onSave={handleSave}
    />
  );
};

// Presentational Component (UI)
const ConfigWizard = ({ config, isValid, onSave }) => {
  return (
    <div>
      {/* UI rendering */}
    </div>
  );
};
```

#### 2. **Compound Component Pattern**
```typescript
// Compound components for complex UI
<Card>
  <CardHeader>
    <CardTitle>Configuration</CardTitle>
  </CardHeader>
  <CardContent>
    <ConfigurationForm />
  </CardContent>
  <CardFooter>
    <ActionButtons />
  </CardFooter>
</Card>
```

#### 3. **Render Props Pattern**
```typescript
// Flexible component composition
<DataProvider>
  {({ data, loading, error }) => (
    <div>
      {loading && <Spinner />}
      {error && <ErrorMessage error={error} />}
      {data && <DataDisplay data={data} />}
    </div>
  )}
</DataProvider>
```

## 🧩 Component Architecture

### Component Hierarchy

```
src/
├── components/
│   ├── ui/                    # Reusable UI primitives
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── dialog.tsx
│   │   └── ...
│   ├── config/                # Configuration-specific components
│   │   ├── ConfigWizard.tsx
│   │   ├── TargetSelector.tsx
│   │   ├── NetworkConfig.tsx
│   │   └── ...
│   ├── layout/                # Layout components
│   │   ├── Header.tsx
│   │   ├── AppSidebar.tsx
│   │   └── ...
│   └── common/                # Shared components
│       ├── LoadingSpinner.tsx
│       ├── ErrorBoundary.tsx
│       └── ...
├── pages/                     # Route components
│   ├── Index.tsx
│   ├── Dashboard.tsx
│   └── ...
├── hooks/                     # Custom hooks
│   ├── useConfiguration.ts
│   ├── useLocalStorage.ts
│   └── ...
└── lib/                       # Utilities
    ├── utils.ts
    ├── validation.ts
    └── ...
```

### Component Design Principles

#### 1. **Single Responsibility**
Each component has one clear purpose:
- `ConfigWizard`: Handles configuration creation/editing
- `TargetSelector`: Manages Kubernetes target selection
- `NetworkConfig`: Configures network settings

#### 2. **Composition over Inheritance**
Components are designed to be composed together:
```typescript
<ConfigWizard>
  <ConfigTabs>
    <TargetTab>
      <TargetSelector />
    </TargetTab>
    <NetworkTab>
      <NetworkConfig />
    </NetworkTab>
  </ConfigTabs>
</ConfigWizard>
```

#### 3. **Props Interface Design**
Clear, typed interfaces for component communication:
```typescript
interface ConfigWizardProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (config: ConfigData) => void;
  existingConfigs?: ConfigData[];
  mode?: 'create' | 'overview';
}
```

#### 4. **Default Props and Optional Parameters**
```typescript
const MyComponent = ({
  title,
  isVisible = true,
  onAction,
  className,
  children
}: MyComponentProps) => {
  // Component implementation
};
```

## 🔄 State Management

### State Architecture

The application uses a hybrid approach to state management:

#### 1. **Local Component State**
```typescript
// Simple local state
const [count, setCount] = useState(0);
const [isLoading, setIsLoading] = useState(false);
```

#### 2. **Custom Hooks for Complex State**
```typescript
// Custom hook for configuration management
const useConfiguration = (initialConfig: ConfigData) => {
  const [config, setConfig] = useState(initialConfig);
  const [isDirty, setIsDirty] = useState(false);
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);
  
  const updateConfig = useCallback((updates: Partial<ConfigData>) => {
    setConfig(prev => ({ ...prev, ...updates }));
    setIsDirty(true);
  }, []);
  
  const validateConfig = useCallback(() => {
    const errors = validateConfiguration(config);
    setValidationErrors(errors);
    return errors.length === 0;
  }, [config]);
  
  return {
    config,
    isDirty,
    validationErrors,
    updateConfig,
    validateConfig
  };
};
```

#### 3. **Context for Global State**
```typescript
// Global application state
interface AppContextType {
  configs: Config[];
  setConfigs: (configs: Config[]) => void;
  activeConfig: Config | null;
  setActiveConfig: (config: Config | null) => void;
}

const AppContext = createContext<AppContextType | undefined>(undefined);
```

#### 4. **LocalStorage for Persistence**
```typescript
// Persistent state with localStorage
const useLocalStorage = <T>(key: string, initialValue: T) => {
  const [storedValue, setStoredValue] = useState<T>(() => {
    try {
      const item = window.localStorage.getItem(key);
      return item ? JSON.parse(item) : initialValue;
    } catch (error) {
      console.error(`Error reading localStorage key "${key}":`, error);
      return initialValue;
    }
  });

  const setValue = (value: T | ((val: T) => T)) => {
    try {
      const valueToStore = value instanceof Function ? value(storedValue) : value;
      setStoredValue(valueToStore);
      window.localStorage.setItem(key, JSON.stringify(valueToStore));
    } catch (error) {
      console.error(`Error setting localStorage key "${key}":`, error);
    }
  };

  return [storedValue, setValue] as const;
};
```

### State Flow Patterns

#### 1. **Unidirectional Data Flow**
```
User Action → Event Handler → State Update → Component Re-render
```

#### 2. **Lifting State Up**
```typescript
// Parent component manages shared state
const Dashboard = () => {
  const [configs, setConfigs] = useState<Config[]>([]);
  const [activeConfig, setActiveConfig] = useState<Config | null>(null);
  
  return (
    <div>
      <ConfigList 
        configs={configs}
        onConfigSelect={setActiveConfig}
      />
      <ConfigEditor
        config={activeConfig}
        onConfigUpdate={setConfigs}
      />
    </div>
  );
};
```

#### 3. **State Normalization**
```typescript
// Normalized state structure
interface NormalizedState {
  configs: {
    byId: Record<string, Config>;
    allIds: string[];
  };
  services: {
    byId: Record<string, Service>;
    allIds: string[];
  };
}
```

## 📊 Data Flow

### Configuration Data Flow

```
User Input → Validation → State Update → JSON Generation → Persistence
     ↓              ↓           ↓            ↓              ↓
Form Fields → Error Display → Component → Config JSON → localStorage
```

#### 1. **Input Processing**
```typescript
// Form input handling
const handleInputChange = (field: string, value: any) => {
  setConfig(prev => ({
    ...prev,
    [field]: value
  }));
  
  // Trigger validation
  validateField(field, value);
};
```

#### 2. **Validation Pipeline**
```typescript
// Multi-level validation
const validateConfiguration = (config: ConfigData): ValidationError[] => {
  const errors: ValidationError[] = [];
  
  // Required field validation
  if (!config.name) {
    errors.push({ field: 'name', message: 'Name is required', type: 'required' });
  }
  
  // Format validation
  if (config.target && !isValidTarget(config.target)) {
    errors.push({ field: 'target', message: 'Invalid target format', type: 'format' });
  }
  
  // Business logic validation
  if (config.network.incoming.enabled && config.network.incoming.ports.length === 0) {
    errors.push({ field: 'ports', message: 'At least one port is required', type: 'required' });
  }
  
  return errors;
};
```

#### 3. **JSON Generation**
```typescript
// Convert internal format to mirrord JSON
const generateConfigJson = (config: ConfigData): string => {
  const configObj: any = {
    target: config.target,
    agent: {},
    feature: {}
  };
  
  // Agent configuration
  if (config.agent.copyTarget) configObj.agent.copy_target = true;
  if (config.agent.scaledown) configObj.agent.scaledown = true;
  
  // Network configuration
  if (config.network.incoming.enabled) {
    configObj.feature.network = {
      incoming: {
        mode: config.network.incoming.mode
      }
    };
  }
  
  return JSON.stringify(configObj, null, 2);
};
```

### Component Communication

#### 1. **Props Down, Events Up**
```typescript
// Parent passes data down
<ConfigWizard 
  config={config}
  onSave={handleSave}
  onClose={handleClose}
/>

// Child emits events up
const ConfigWizard = ({ config, onSave, onClose }) => {
  const handleSave = () => {
    onSave(updatedConfig);
  };
  
  return (
    <div>
      <button onClick={handleSave}>Save</button>
      <button onClick={onClose}>Close</button>
    </div>
  );
};
```

#### 2. **Context for Deep Prop Drilling**
```typescript
// Avoid prop drilling with context
const ConfigContext = createContext<ConfigContextType | undefined>(undefined);

const useConfig = () => {
  const context = useContext(ConfigContext);
  if (!context) {
    throw new Error('useConfig must be used within ConfigProvider');
  }
  return context;
};
```

## 🎨 Design System

### Design Tokens

#### 1. **Color System**
```css
:root {
  /* Primary Colors */
  --primary: 258 90% 66%;
  --primary-foreground: 240 5% 98%;
  
  /* Semantic Colors */
  --background: 240 5% 98%;
  --foreground: 240 6% 7%;
  --muted: 240 4% 88%;
  --muted-foreground: 240 5% 35%;
  
  /* Status Colors */
  --destructive: 0 76% 60%;
  --destructive-foreground: 240 5% 98%;
  
  /* Custom Gradients */
  --gradient-primary: linear-gradient(135deg, hsl(258 90% 66%), hsl(272 91% 70%));
  --gradient-card: linear-gradient(145deg, hsl(240 5% 96%), hsl(240 6% 90%));
}
```

#### 2. **Typography Scale**
```css
/* Font Sizes */
--text-xs: 0.75rem;    /* 12px */
--text-sm: 0.875rem;   /* 14px */
--text-base: 1rem;     /* 16px */
--text-lg: 1.125rem;   /* 18px */
--text-xl: 1.25rem;    /* 20px */
--text-2xl: 1.5rem;    /* 24px */
--text-3xl: 1.875rem;  /* 30px */
--text-4xl: 2.25rem;   /* 36px */

/* Font Weights */
--font-normal: 400;
--font-medium: 500;
--font-semibold: 600;
--font-bold: 700;
```

#### 3. **Spacing System**
```css
/* Spacing Scale (based on 4px grid) */
--space-1: 0.25rem;   /* 4px */
--space-2: 0.5rem;    /* 8px */
--space-3: 0.75rem;   /* 12px */
--space-4: 1rem;      /* 16px */
--space-6: 1.5rem;    /* 24px */
--space-8: 2rem;      /* 32px */
--space-12: 3rem;     /* 48px */
--space-16: 4rem;     /* 64px */
```

### Component Design Patterns

#### 1. **Variant System**
```typescript
// Component variants using class-variance-authority
const buttonVariants = cva(
  "inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        destructive: "bg-destructive text-destructive-foreground hover:bg-destructive/90",
        outline: "border border-input hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        icon: "h-10 w-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
);
```

#### 2. **Composition Patterns**
```typescript
// Compound components
const Card = ({ children, className, ...props }) => (
  <div className={cn("rounded-lg border bg-card text-card-foreground shadow-sm", className)} {...props}>
    {children}
  </div>
);

const CardHeader = ({ className, ...props }) => (
  <div className={cn("flex flex-col space-y-1.5 p-6", className)} {...props} />
);

const CardTitle = ({ className, ...props }) => (
  <h3 className={cn("text-2xl font-semibold leading-none tracking-tight", className)} {...props} />
);

const CardContent = ({ className, ...props }) => (
  <div className={cn("p-6 pt-0", className)} {...props} />
);
```

### Responsive Design

#### 1. **Breakpoint System**
```typescript
// Tailwind CSS breakpoints
const breakpoints = {
  sm: '640px',   // Small devices
  md: '768px',   // Medium devices
  lg: '1024px',  // Large devices
  xl: '1280px',  // Extra large devices
  '2xl': '1536px' // 2X large devices
};
```

#### 2. **Responsive Patterns**
```tsx
// Mobile-first responsive design
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
  {/* Responsive grid */}
</div>

// Responsive typography
<h1 className="text-2xl md:text-3xl lg:text-4xl font-bold">
  Responsive Title
</h1>

// Responsive spacing
<div className="p-4 md:p-6 lg:p-8">
  {/* Responsive padding */}
</div>
```

## ⚡ Performance Architecture

### Bundle Optimization

#### 1. **Code Splitting**
```typescript
// Route-based code splitting
const Dashboard = lazy(() => import('./pages/Dashboard'));
const ConfigWizard = lazy(() => import('./components/ConfigWizard'));

// Component-based code splitting
const HeavyComponent = lazy(() => import('./HeavyComponent'));
```

#### 2. **Tree Shaking**
```typescript
// Import only what you need
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';

// Avoid default imports for large libraries
import { debounce } from 'lodash/debounce';
```

#### 3. **Asset Optimization**
```typescript
// Vite configuration for asset optimization
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          ui: ['@radix-ui/react-dialog', '@radix-ui/react-select'],
          utils: ['clsx', 'tailwind-merge']
        }
      }
    }
  }
});
```

### Runtime Performance

#### 1. **Memoization Strategy**
```typescript
// Memoize expensive calculations
const expensiveValue = useMemo(() => {
  return computeExpensiveValue(data);
}, [data]);

// Memoize component props
const memoizedProps = useMemo(() => ({
  config: config,
  onSave: handleSave
}), [config, handleSave]);

// Memoize callback functions
const handleClick = useCallback((id: string) => {
  onItemClick(id);
}, [onItemClick]);
```

#### 2. **Virtual Scrolling**
```typescript
// For large lists
import { FixedSizeList as List } from 'react-window';

const VirtualizedList = ({ items }) => (
  <List
    height={600}
    itemCount={items.length}
    itemSize={50}
    itemData={items}
  >
    {({ index, style, data }) => (
      <div style={style}>
        {data[index].name}
      </div>
    )}
  </List>
);
```

#### 3. **Lazy Loading**
```typescript
// Lazy load images
const LazyImage = ({ src, alt, ...props }) => {
  const [isLoaded, setIsLoaded] = useState(false);
  
  return (
    <img
      src={isLoaded ? src : placeholder}
      alt={alt}
      onLoad={() => setIsLoaded(true)}
      {...props}
    />
  );
};
```

### Caching Strategy

#### 1. **Local Storage Caching**
```typescript
// Cache configuration data
const useConfigCache = () => {
  const [cache, setCache] = useState<Map<string, any>>(new Map());
  
  const get = useCallback((key: string) => {
    if (cache.has(key)) {
      return cache.get(key);
    }
    
    const item = localStorage.getItem(key);
    if (item) {
      const parsed = JSON.parse(item);
      setCache(prev => new Map(prev).set(key, parsed));
      return parsed;
    }
    
    return null;
  }, [cache]);
  
  const set = useCallback((key: string, value: any) => {
    setCache(prev => new Map(prev).set(key, value));
    localStorage.setItem(key, JSON.stringify(value));
  }, []);
  
  return { get, set };
};
```

#### 2. **Memory Caching**
```typescript
// Simple in-memory cache
class MemoryCache {
  private cache = new Map<string, { value: any; timestamp: number }>();
  private ttl = 5 * 60 * 1000; // 5 minutes
  
  get(key: string) {
    const item = this.cache.get(key);
    if (!item) return null;
    
    if (Date.now() - item.timestamp > this.ttl) {
      this.cache.delete(key);
      return null;
    }
    
    return item.value;
  }
  
  set(key: string, value: any) {
    this.cache.set(key, {
      value,
      timestamp: Date.now()
    });
  }
}
```

## 🔒 Security Considerations

### Input Validation

#### 1. **Client-Side Validation**
```typescript
// Validate all user inputs
const validateInput = (input: string, type: 'email' | 'url' | 'json') => {
  switch (type) {
    case 'email':
      return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(input);
    case 'url':
      try {
        new URL(input);
        return true;
      } catch {
        return false;
      }
    case 'json':
      try {
        JSON.parse(input);
        return true;
      } catch {
        return false;
      }
    default:
      return false;
  }
};
```

#### 2. **XSS Prevention**
```typescript
// Sanitize user input
const sanitizeInput = (input: string) => {
  return input
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;');
};
```

### Data Protection

#### 1. **Sensitive Data Handling**
```typescript
// Don't store sensitive data in localStorage
const secureStorage = {
  set: (key: string, value: any) => {
    // Only store non-sensitive configuration data
    if (isSensitiveKey(key)) {
      throw new Error('Sensitive data cannot be stored in localStorage');
    }
    localStorage.setItem(key, JSON.stringify(value));
  },
  
  get: (key: string) => {
    if (isSensitiveKey(key)) {
      return null;
    }
    return JSON.parse(localStorage.getItem(key) || 'null');
  }
};
```

#### 2. **Content Security Policy**
```html
<!-- CSP header for production -->
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self'; 
               script-src 'self' 'unsafe-inline'; 
               style-src 'self' 'unsafe-inline';">
```

## 📈 Scalability

### Component Scalability

#### 1. **Modular Architecture**
```typescript
// Feature-based organization
src/
├── features/
│   ├── configuration/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── types/
│   │   └── utils/
│   ├── dashboard/
│   │   ├── components/
│   │   ├── hooks/
│   │   └── types/
│   └── onboarding/
│       ├── components/
│       └── hooks/
```

#### 2. **Plugin Architecture**
```typescript
// Extensible configuration system
interface ConfigPlugin {
  name: string;
  validate: (config: ConfigData) => ValidationError[];
  transform: (config: ConfigData) => ConfigData;
  render: (props: any) => React.ReactNode;
}

const pluginRegistry = new Map<string, ConfigPlugin>();

const registerPlugin = (plugin: ConfigPlugin) => {
  pluginRegistry.set(plugin.name, plugin);
};
```

### Performance Scalability

#### 1. **Lazy Loading Strategy**
```typescript
// Progressive loading
const ProgressiveLoader = () => {
  const [loadedModules, setLoadedModules] = useState<Set<string>>(new Set());
  
  const loadModule = useCallback(async (moduleName: string) => {
    if (loadedModules.has(moduleName)) return;
    
    const module = await import(`./modules/${moduleName}`);
    setLoadedModules(prev => new Set(prev).add(moduleName));
    return module;
  }, [loadedModules]);
  
  return { loadModule, loadedModules };
};
```

#### 2. **State Management Scalability**
```typescript
// Redux-like state management for complex applications
interface State {
  configs: ConfigState;
  ui: UIState;
  user: UserState;
}

const useStore = () => {
  const [state, setState] = useState<State>(initialState);
  
  const dispatch = useCallback((action: Action) => {
    setState(prevState => reducer(prevState, action));
  }, []);
  
  return { state, dispatch };
};
```

### Team Scalability

#### 1. **Code Organization**
```typescript
// Clear module boundaries
// Each feature is self-contained
// Shared utilities are clearly identified
// Type definitions are centralized
```

#### 2. **Development Workflow**
```typescript
// Consistent coding standards
// Automated testing
// Code review process
// Documentation requirements
```

This architecture documentation provides a comprehensive overview of the mirrord Configuration Wizard Frontend's design, patterns, and scalability considerations, serving as a guide for developers and architects working on the system.
