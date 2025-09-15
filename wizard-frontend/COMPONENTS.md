# Component Documentation

This document provides detailed documentation for all major components in the mirrord Configuration Wizard Frontend.

## 📋 Table of Contents

- [Core Components](#core-components)
- [Configuration Components](#configuration-components)
- [UI Components](#ui-components)
- [Page Components](#page-components)
- [Utility Components](#utility-components)

## 🏗️ Core Components

### ConfigWizard

The main configuration wizard component that guides users through creating mirrord configurations.

**Location**: `src/components/ConfigWizard.tsx`

**Purpose**: Provides a multi-step interface for creating and editing mirrord configurations with real-time JSON generation.

**Props**:
```typescript
interface ConfigWizardProps {
  isOpen: boolean;                    // Controls wizard visibility
  onClose: () => void;               // Callback when wizard is closed
  onSave: (config: ConfigData) => void; // Callback when config is saved
  existingConfigs?: ConfigData[];    // Existing configurations for reference
  mode?: 'create' | 'overview';      // Wizard mode
}
```

**Features**:
- Multi-step configuration process (Target → Network → Export)
- Support for different configuration modes (steal, mirror, replace)
- Real-time JSON validation and generation
- Port mapping configuration
- HTTP filtering (headers and paths)
- Export functionality (copy to clipboard, download)

**Usage Example**:
```tsx
import { ConfigWizard } from '@/components/ConfigWizard';

function MyComponent() {
  const [showWizard, setShowWizard] = useState(false);
  
  const handleSave = (config: ConfigData) => {
    console.log('New configuration:', config);
    setShowWizard(false);
  };

  return (
    <ConfigWizard
      isOpen={showWizard}
      onClose={() => setShowWizard(false)}
      onSave={handleSave}
      mode="create"
    />
  );
}
```

**Configuration Flow**:
1. **Target Selection**: Choose Kubernetes namespace, resource type, and target
2. **Network Configuration**: Configure traffic filtering and port mappings
3. **Export**: Review and export the generated JSON configuration

### Dashboard

The main dashboard component for managing configurations.

**Location**: `src/components/Dashboard.tsx`

**Purpose**: Provides the main interface for viewing and managing mirrord configurations.

**Features**:
- Configuration overview and management
- Service grouping
- Active configuration switching
- Quick access to configuration wizard
- Configuration CRUD operations

**State Management**:
- Uses localStorage for persistence
- Groups configurations by service
- Manages active configuration state

**Usage Example**:
```tsx
import { Dashboard } from '@/components/Dashboard';

function App() {
  return (
    <SidebarProvider>
      <Dashboard />
    </SidebarProvider>
  );
}
```

### Header

Application header component with branding and version information.

**Location**: `src/components/Header.tsx`

**Purpose**: Displays the application header with mirrord branding and version.

**Features**:
- mirrord logo and branding
- Version badge
- Sidebar trigger for mobile

**Usage Example**:
```tsx
import { Header } from '@/components/Header';

function Layout() {
  return (
    <div>
      <Header />
      {/* Main content */}
    </div>
  );
}
```

### AppSidebar

Navigation sidebar component.

**Location**: `src/components/AppSidebar.tsx`

**Purpose**: Provides navigation between different sections and displays saved configurations.

**Props**:
```typescript
interface AppSidebarProps {
  services: Service[];                    // Grouped configurations by service
  activeSection: string;                 // Currently active section
  onSectionChange: (section: string) => void; // Section change callback
}
```

**Features**:
- Main navigation (Home, Config Files)
- Saved configurations list
- Active configuration indicators
- Service grouping

## 🔧 Configuration Components

### TargetSelector

Component for selecting Kubernetes targets.

**Location**: `src/components/config/TargetSelector.tsx`

**Purpose**: Handles the selection of Kubernetes namespaces, resource types, and specific targets.

**Features**:
- Namespace selection dropdown
- Resource type selection (Deployment, StatefulSet, etc.)
- Target search and selection
- Configuration naming

### NetworkConfig

Network configuration component.

**Location**: `src/components/config/NetworkConfig.tsx`

**Purpose**: Configures network-related settings including traffic filtering and port mappings.

**Features**:
- Incoming traffic configuration (steal/mirror modes)
- HTTP filtering (headers and paths)
- Port mapping setup
- Outgoing traffic configuration
- DNS settings

### PortConfig

Port configuration component.

**Location**: `src/components/config/PortConfig.tsx`

**Purpose**: Handles port mapping between local and remote services.

**Features**:
- Port detection and selection
- Local to remote port mapping
- Port-specific configuration options

### ConfigTabs

Tabbed interface for configuration sections.

**Location**: `src/components/config/ConfigTabs.tsx`

**Purpose**: Provides tabbed navigation between different configuration sections.

**Features**:
- Tab-based navigation
- Section validation
- Progress indication

### ConfigExport

Configuration export component.

**Location**: `src/components/config/ConfigExport.tsx`

**Purpose**: Handles the export and preview of generated configurations.

**Features**:
- JSON preview and editing
- Copy to clipboard functionality
- File download
- JSON validation

### BoilerplateSelector

Component for selecting configuration templates.

**Location**: `src/components/config/BoilerplateSelector.tsx`

**Purpose**: Allows users to start with pre-configured templates.

**Features**:
- Template selection (steal, mirror, replace modes)
- Template descriptions and features
- Quick configuration setup

## 🎨 UI Components

The application uses shadcn/ui components built on Radix UI primitives. All components are located in `src/components/ui/`.

### Key UI Components

#### Button
```tsx
import { Button } from '@/components/ui/button';

<Button variant="default" size="sm">
  Click me
</Button>
```

#### Card
```tsx
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

<Card>
  <CardHeader>
    <CardTitle>Card Title</CardTitle>
  </CardHeader>
  <CardContent>
    Card content goes here
  </CardContent>
</Card>
```

#### Dialog
```tsx
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';

<Dialog open={isOpen} onOpenChange={setIsOpen}>
  <DialogContent>
    <DialogHeader>
      <DialogTitle>Dialog Title</DialogTitle>
    </DialogHeader>
    {/* Dialog content */}
  </DialogContent>
</Dialog>
```

#### Form Components
```tsx
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

<div>
  <Label htmlFor="input">Label</Label>
  <Input id="input" placeholder="Enter value" />
</div>

<Select>
  <SelectTrigger>
    <SelectValue placeholder="Select option" />
  </SelectTrigger>
  <SelectContent>
    <SelectItem value="option1">Option 1</SelectItem>
    <SelectItem value="option2">Option 2</SelectItem>
  </SelectContent>
</Select>
```

## 📄 Page Components

### Index

Landing page component with onboarding flow.

**Location**: `src/pages/Index.tsx`

**Purpose**: Handles the initial landing experience and redirects users based on their state.

**Features**:
- Onboarding completion check
- Automatic redirection to dashboard
- Onboarding component integration

### Dashboard Page

Main dashboard page component.

**Location**: `src/pages/Dashboard.tsx`

**Purpose**: Wraps the Dashboard component with routing and layout.

**Features**:
- Sidebar layout
- Configuration management
- Navigation between sections

### Onboarding

Interactive onboarding flow component.

**Location**: `src/pages/Onboarding.tsx`

**Purpose**: Guides new users through understanding mirrord and creating their first configuration.

**Steps**:
1. **Welcome**: Introduction to mirrord
2. **What is mirrord?**: Core concepts explanation
3. **How mirrord Works**: Architecture overview
4. **Filtering Mode**: Steal mode explanation
5. **Mirror Mode**: Mirror mode explanation
6. **Replace Mode**: Replace mode explanation
7. **Feedback Loop**: Development workflow benefits
8. **Configuration**: Create first configuration

**Features**:
- Step-by-step progression
- Visual explanations with diagrams
- Skip options for experienced users
- Integration with configuration wizard

### StyleGuide

Component style guide page.

**Location**: `src/pages/StyleGuide.tsx`

**Purpose**: Displays all UI components for development and design reference.

## 🛠️ Utility Components

### Hooks

#### useToast
Custom hook for toast notifications.

**Location**: `src/hooks/use-toast.ts`

**Usage**:
```tsx
import { useToast } from '@/hooks/use-toast';

function MyComponent() {
  const { toast } = useToast();
  
  const handleClick = () => {
    toast({
      title: "Success",
      description: "Configuration saved successfully",
    });
  };
}
```

#### useMobile
Custom hook for mobile detection.

**Location**: `src/hooks/use-mobile.tsx`

**Usage**:
```tsx
import { useMobile } from '@/hooks/use-mobile';

function MyComponent() {
  const isMobile = useMobile();
  
  return (
    <div className={isMobile ? 'mobile-layout' : 'desktop-layout'}>
      Content
    </div>
  );
}
```

### Utilities

#### cn
Utility function for combining class names.

**Location**: `src/lib/utils.ts`

**Usage**:
```tsx
import { cn } from '@/lib/utils';

function MyComponent({ className, ...props }) {
  return (
    <div className={cn("base-classes", className)} {...props}>
      Content
    </div>
  );
}
```

## 🎯 Component Patterns

### State Management
- Use React hooks for local state
- localStorage for persistence
- Props for parent-child communication
- Context for global state when needed

### Styling
- Tailwind CSS for styling
- Custom CSS variables for theming
- Responsive design patterns
- Component variants using class-variance-authority

### TypeScript
- Strict typing for all props
- Interface definitions for data structures
- Generic components where appropriate
- Proper error handling

### Accessibility
- Semantic HTML elements
- ARIA attributes where needed
- Keyboard navigation support
- Screen reader compatibility

## 🔄 Component Lifecycle

### Configuration Wizard Flow
1. **Initialization**: Set up default configuration state
2. **Target Selection**: User selects Kubernetes target
3. **Network Configuration**: User configures traffic settings
4. **Validation**: Real-time validation of configuration
5. **Export**: Generate and export JSON configuration
6. **Save**: Persist configuration to localStorage

### Dashboard Management
1. **Load**: Retrieve configurations from localStorage
2. **Group**: Organize configurations by service
3. **Display**: Render configuration cards
4. **Interact**: Handle CRUD operations
5. **Persist**: Update localStorage with changes

## 🧪 Testing Components

### Component Testing
- Test component rendering
- Test user interactions
- Test state changes
- Test prop handling

### Integration Testing
- Test component communication
- Test data flow
- Test localStorage integration
- Test routing

### Visual Testing
- Test responsive design
- Test theme switching
- Test component variants
- Test accessibility

## 📝 Best Practices

### Component Design
- Single responsibility principle
- Reusable and composable
- Clear prop interfaces
- Proper TypeScript typing

### Performance
- Memoization where appropriate
- Lazy loading for heavy components
- Efficient re-renders
- Optimized bundle size

### Maintainability
- Clear component structure
- Comprehensive documentation
- Consistent naming conventions
- Proper error boundaries

### User Experience
- Intuitive interfaces
- Clear feedback
- Responsive design
- Accessibility compliance
