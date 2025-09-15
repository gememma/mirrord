# Development Guide

This guide provides comprehensive information for developers working on the mirrord Configuration Wizard Frontend.

## 📋 Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Code Style and Standards](#code-style-and-standards)
- [Component Development](#component-development)
- [State Management](#state-management)
- [Testing](#testing)
- [Build and Deployment](#build-and-deployment)
- [Troubleshooting](#troubleshooting)

## 🚀 Getting Started

### Prerequisites

- **Node.js**: Version 18 or higher
- **npm**: Version 8 or higher (or yarn)
- **Git**: For version control
- **VS Code**: Recommended IDE with extensions

### Required VS Code Extensions

```json
{
  "recommendations": [
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "ms-vscode.vscode-typescript-next",
    "formulahendry.auto-rename-tag",
    "christian-kohler.path-intellisense",
    "ms-vscode.vscode-eslint"
  ]
}
```

### Initial Setup

1. **Clone and navigate to the project**
   ```bash
   git clone <repository-url>
   cd mirrord/wizard-frontend
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Start development server**
   ```bash
   npm run dev
   ```

4. **Open in browser**
   Navigate to `http://localhost:8080`

## 🛠️ Development Environment

### Project Structure

```
wizard-frontend/
├── src/
│   ├── components/          # React components
│   │   ├── ui/             # shadcn/ui components
│   │   ├── config/         # Configuration-specific components
│   │   └── ...             # Other components
│   ├── pages/              # Route components
│   ├── hooks/              # Custom React hooks
│   ├── lib/                # Utility functions
│   ├── types/              # TypeScript type definitions
│   └── assets/             # Static assets
├── public/                 # Public static files
├── dist/                   # Built application
├── node_modules/           # Dependencies
├── package.json            # Project configuration
├── tsconfig.json           # TypeScript configuration
├── tailwind.config.ts      # Tailwind CSS configuration
├── vite.config.ts          # Vite configuration
└── eslint.config.js        # ESLint configuration
```

### Configuration Files

#### TypeScript Configuration
- `tsconfig.json`: Main TypeScript configuration
- `tsconfig.app.json`: Application-specific TypeScript settings
- `tsconfig.node.json`: Node.js-specific TypeScript settings

#### Build Configuration
- `vite.config.ts`: Vite build tool configuration
- `tailwind.config.ts`: Tailwind CSS configuration
- `postcss.config.js`: PostCSS configuration

#### Code Quality
- `eslint.config.js`: ESLint configuration
- `.prettierrc`: Prettier formatting configuration

## 📝 Code Style and Standards

### TypeScript Guidelines

#### Type Definitions
```typescript
// Use interfaces for object shapes
interface ConfigData {
  name: string;
  target: string;
  // ... other properties
}

// Use types for unions and computed types
type ConfigMode = 'create' | 'overview';
type ValidationError = {
  field: string;
  message: string;
  type: 'required' | 'invalid' | 'format';
};

// Use enums for constants
enum ConfigStatus {
  ACTIVE = 'active',
  INACTIVE = 'inactive',
  DRAFT = 'draft'
}
```

#### Component Props
```typescript
// Always define prop interfaces
interface MyComponentProps {
  title: string;
  isVisible: boolean;
  onAction: (id: string) => void;
  children?: React.ReactNode;
}

// Use default props when appropriate
const MyComponent: React.FC<MyComponentProps> = ({
  title,
  isVisible = true,
  onAction,
  children
}) => {
  // Component implementation
};
```

#### Event Handlers
```typescript
// Use specific event types
const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
  setValue(event.target.value);
};

const handleFormSubmit = (event: React.FormEvent<HTMLFormElement>) => {
  event.preventDefault();
  // Handle form submission
};

// Use callback refs for DOM manipulation
const inputRef = useCallback((node: HTMLInputElement | null) => {
  if (node) {
    node.focus();
  }
}, []);
```

### React Best Practices

#### Component Structure
```typescript
// 1. Imports (external libraries first, then internal)
import React, { useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { MyComponent } from './MyComponent';

// 2. Type definitions
interface Props {
  // ... prop types
}

// 3. Component definition
export const MyComponent: React.FC<Props> = ({ prop1, prop2 }) => {
  // 4. Hooks (useState, useEffect, etc.)
  const [state, setState] = useState(initialState);
  
  // 5. Event handlers
  const handleClick = useCallback(() => {
    // Handle click
  }, [dependencies]);
  
  // 6. Effects
  useEffect(() => {
    // Side effects
  }, [dependencies]);
  
  // 7. Render
  return (
    <div>
      {/* JSX content */}
    </div>
  );
};
```

#### State Management
```typescript
// Use useState for local state
const [count, setCount] = useState(0);

// Use useReducer for complex state
const [state, dispatch] = useReducer(reducer, initialState);

// Use useMemo for expensive calculations
const expensiveValue = useMemo(() => {
  return computeExpensiveValue(data);
}, [data]);

// Use useCallback for stable function references
const stableCallback = useCallback((id: string) => {
  handleAction(id);
}, [handleAction]);
```

#### Custom Hooks
```typescript
// Custom hook for configuration management
const useConfiguration = (initialConfig: ConfigData) => {
  const [config, setConfig] = useState(initialConfig);
  const [isDirty, setIsDirty] = useState(false);
  
  const updateConfig = useCallback((updates: Partial<ConfigData>) => {
    setConfig(prev => ({ ...prev, ...updates }));
    setIsDirty(true);
  }, []);
  
  const resetConfig = useCallback(() => {
    setConfig(initialConfig);
    setIsDirty(false);
  }, [initialConfig]);
  
  return {
    config,
    isDirty,
    updateConfig,
    resetConfig
  };
};
```

### Styling Guidelines

#### Tailwind CSS Classes
```tsx
// Use semantic class names
<div className="flex items-center justify-between p-4 bg-card rounded-lg border">
  <h2 className="text-lg font-semibold text-foreground">Title</h2>
  <Button variant="outline" size="sm">Action</Button>
</div>

// Use responsive design
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
  {/* Responsive grid */}
</div>

// Use custom CSS variables
<div className="bg-gradient-primary text-primary-foreground">
  {/* Custom gradient */}
</div>
```

#### Component Styling
```typescript
// Use cn utility for conditional classes
import { cn } from '@/lib/utils';

const MyComponent = ({ className, isActive, ...props }) => {
  return (
    <div
      className={cn(
        "base-classes",
        isActive && "active-classes",
        className
      )}
      {...props}
    />
  );
};
```

## 🧩 Component Development

### Creating New Components

#### 1. Component File Structure
```
src/components/MyComponent/
├── index.ts              # Export file
├── MyComponent.tsx       # Main component
├── MyComponent.test.tsx  # Tests
└── MyComponent.stories.tsx # Storybook stories
```

#### 2. Component Template
```typescript
// MyComponent.tsx
import React from 'react';
import { cn } from '@/lib/utils';

interface MyComponentProps {
  title: string;
  isVisible?: boolean;
  onAction?: (id: string) => void;
  className?: string;
  children?: React.ReactNode;
}

export const MyComponent: React.FC<MyComponentProps> = ({
  title,
  isVisible = true,
  onAction,
  className,
  children
}) => {
  if (!isVisible) return null;

  return (
    <div className={cn("my-component", className)}>
      <h2 className="text-lg font-semibold">{title}</h2>
      {children}
      {onAction && (
        <button onClick={() => onAction('test')}>
          Action
        </button>
      )}
    </div>
  );
};

// index.ts
export { MyComponent } from './MyComponent';
export type { MyComponentProps } from './MyComponent';
```

#### 3. Component Testing
```typescript
// MyComponent.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { MyComponent } from './MyComponent';

describe('MyComponent', () => {
  it('renders with title', () => {
    render(<MyComponent title="Test Title" />);
    expect(screen.getByText('Test Title')).toBeInTheDocument();
  });

  it('calls onAction when button is clicked', () => {
    const mockAction = jest.fn();
    render(<MyComponent title="Test" onAction={mockAction} />);
    
    fireEvent.click(screen.getByText('Action'));
    expect(mockAction).toHaveBeenCalledWith('test');
  });

  it('does not render when isVisible is false', () => {
    render(<MyComponent title="Test" isVisible={false} />);
    expect(screen.queryByText('Test')).not.toBeInTheDocument();
  });
});
```

### Component Patterns

#### Compound Components
```typescript
// Card compound component
interface CardProps {
  children: React.ReactNode;
  className?: string;
}

interface CardHeaderProps {
  children: React.ReactNode;
  className?: string;
}

interface CardContentProps {
  children: React.ReactNode;
  className?: string;
}

const Card = ({ children, className }: CardProps) => (
  <div className={cn("card", className)}>{children}</div>
);

const CardHeader = ({ children, className }: CardHeaderProps) => (
  <div className={cn("card-header", className)}>{children}</div>
);

const CardContent = ({ children, className }: CardContentProps) => (
  <div className={cn("card-content", className)}>{children}</div>
);

// Usage
<Card>
  <CardHeader>Title</CardHeader>
  <CardContent>Content</CardContent>
</Card>
```

#### Render Props Pattern
```typescript
interface DataProviderProps {
  children: (data: any, loading: boolean, error: string | null) => React.ReactNode;
}

const DataProvider: React.FC<DataProviderProps> = ({ children }) => {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Fetch data logic

  return <>{children(data, loading, error)}</>;
};

// Usage
<DataProvider>
  {(data, loading, error) => (
    <div>
      {loading && <div>Loading...</div>}
      {error && <div>Error: {error}</div>}
      {data && <div>Data: {JSON.stringify(data)}</div>}
    </div>
  )}
</DataProvider>
```

## 🔄 State Management

### Local State
```typescript
// Simple state
const [count, setCount] = useState(0);

// Complex state with useReducer
interface State {
  configs: Config[];
  loading: boolean;
  error: string | null;
}

type Action = 
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_CONFIGS'; payload: Config[] }
  | { type: 'SET_ERROR'; payload: string | null };

const reducer = (state: State, action: Action): State => {
  switch (action.type) {
    case 'SET_LOADING':
      return { ...state, loading: action.payload };
    case 'SET_CONFIGS':
      return { ...state, configs: action.payload, loading: false };
    case 'SET_ERROR':
      return { ...state, error: action.payload, loading: false };
    default:
      return state;
  }
};

const [state, dispatch] = useReducer(reducer, initialState);
```

### Global State
```typescript
// Context for global state
interface AppContextType {
  configs: Config[];
  setConfigs: (configs: Config[]) => void;
  activeConfig: Config | null;
  setActiveConfig: (config: Config | null) => void;
}

const AppContext = createContext<AppContextType | undefined>(undefined);

export const useAppContext = () => {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useAppContext must be used within AppProvider');
  }
  return context;
};

// Provider component
export const AppProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [configs, setConfigs] = useState<Config[]>([]);
  const [activeConfig, setActiveConfig] = useState<Config | null>(null);

  const value = {
    configs,
    setConfigs,
    activeConfig,
    setActiveConfig
  };

  return (
    <AppContext.Provider value={value}>
      {children}
    </AppContext.Provider>
  );
};
```

### Data Persistence
```typescript
// Custom hook for localStorage
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

// Usage
const [configs, setConfigs] = useLocalStorage<Config[]>('mirrord-configs', []);
```

## 🧪 Testing

### Testing Setup
```typescript
// test-utils.tsx
import React from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

const createTestQueryClient = () => new QueryClient({
  defaultOptions: {
    queries: { retry: false },
    mutations: { retry: false }
  }
});

interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  queryClient?: QueryClient;
}

const customRender = (
  ui: React.ReactElement,
  { queryClient = createTestQueryClient(), ...renderOptions }: CustomRenderOptions = {}
) => {
  const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );

  return render(ui, { wrapper: Wrapper, ...renderOptions });
};

export * from '@testing-library/react';
export { customRender as render };
```

### Component Testing
```typescript
// Component test example
import { render, screen, fireEvent, waitFor } from '../test-utils';
import { ConfigWizard } from './ConfigWizard';

describe('ConfigWizard', () => {
  const mockOnSave = jest.fn();
  const mockOnClose = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('renders wizard when open', () => {
    render(
      <ConfigWizard
        isOpen={true}
        onClose={mockOnClose}
        onSave={mockOnSave}
      />
    );

    expect(screen.getByText('Configuration Setup')).toBeInTheDocument();
  });

  it('calls onClose when close button is clicked', () => {
    render(
      <ConfigWizard
        isOpen={true}
        onClose={mockOnClose}
        onSave={mockOnSave}
      />
    );

    fireEvent.click(screen.getByRole('button', { name: /close/i }));
    expect(mockOnClose).toHaveBeenCalled();
  });

  it('validates required fields', async () => {
    render(
      <ConfigWizard
        isOpen={true}
        onClose={mockOnClose}
        onSave={mockOnSave}
      />
    );

    // Try to save without filling required fields
    fireEvent.click(screen.getByText('Save & Set Active'));
    
    await waitFor(() => {
      expect(screen.getByText('Please select a target to continue')).toBeInTheDocument();
    });
  });
});
```

### Integration Testing
```typescript
// Integration test example
import { render, screen, fireEvent, waitFor } from '../test-utils';
import { Dashboard } from './Dashboard';

describe('Dashboard Integration', () => {
  it('creates and saves a new configuration', async () => {
    render(<Dashboard />);

    // Open wizard
    fireEvent.click(screen.getByText('Create Configuration'));

    // Fill in configuration
    fireEvent.change(screen.getByLabelText('Configuration Name'), {
      target: { value: 'Test Config' }
    });

    // Select target
    fireEvent.click(screen.getByText('Search for target...'));
    fireEvent.click(screen.getByText('api-service'));

    // Save configuration
    fireEvent.click(screen.getByText('Save & Set Active'));

    await waitFor(() => {
      expect(screen.getByText('Test Config')).toBeInTheDocument();
    });
  });
});
```

## 🚀 Build and Deployment

### Development Build
```bash
# Start development server
npm run dev

# Build for development
npm run build:dev
```

### Production Build
```bash
# Build for production
npm run build

# Preview production build
npm run preview
```

### Build Optimization
```typescript
// vite.config.ts optimizations
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          ui: ['@radix-ui/react-dialog', '@radix-ui/react-select'],
        }
      }
    },
    chunkSizeWarningLimit: 1000,
  },
  optimizeDeps: {
    include: ['react', 'react-dom', 'react-router-dom']
  }
});
```

### Environment Variables
```bash
# .env.local
VITE_API_URL=http://localhost:3000
VITE_APP_VERSION=1.0.0
VITE_DEBUG=true
```

```typescript
// Use environment variables
const apiUrl = import.meta.env.VITE_API_URL;
const isDebug = import.meta.env.VITE_DEBUG === 'true';
```

## 🔧 Troubleshooting

### Common Issues

#### TypeScript Errors
```bash
# Clear TypeScript cache
rm -rf node_modules/.cache
npm run build

# Check TypeScript configuration
npx tsc --noEmit
```

#### Build Errors
```bash
# Clear all caches
rm -rf node_modules
rm -rf dist
npm install
npm run build
```

#### Development Server Issues
```bash
# Clear Vite cache
rm -rf node_modules/.vite
npm run dev

# Check port availability
lsof -ti:8080
```

### Debugging

#### React DevTools
- Install React Developer Tools browser extension
- Use Profiler to identify performance issues
- Check component state and props

#### Console Debugging
```typescript
// Add debug logging
const debugLog = (message: string, data?: any) => {
  if (process.env.NODE_ENV === 'development') {
    console.log(`[DEBUG] ${message}`, data);
  }
};

// Use in components
const MyComponent = () => {
  const [state, setState] = useState(initialState);
  
  useEffect(() => {
    debugLog('Component mounted', { state });
  }, [state]);
};
```

#### Error Boundaries
```typescript
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error?: Error }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary">
          <h2>Something went wrong.</h2>
          <details>
            {this.state.error?.message}
          </details>
        </div>
      );
    }

    return this.props.children;
  }
}
```

### Performance Optimization

#### Bundle Analysis
```bash
# Analyze bundle size
npm run build
npx vite-bundle-analyzer dist/assets/*.js
```

#### Code Splitting
```typescript
// Lazy load components
const ConfigWizard = lazy(() => import('./ConfigWizard'));
const Dashboard = lazy(() => import('./Dashboard'));

// Use Suspense
<Suspense fallback={<div>Loading...</div>}>
  <ConfigWizard />
</Suspense>
```

#### Memoization
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
```

This development guide provides comprehensive information for developers working on the mirrord Configuration Wizard Frontend, covering everything from setup to advanced patterns and troubleshooting.
