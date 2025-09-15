import React from 'react';
import { Wizard, WizardStep, useWizardData } from './Wizard';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

// Example step components
const PersonalInfoStep = ({ data, updateData }: { data: any; updateData: (data: any) => void }) => (
  <div className="space-y-4">
    <div>
      <Label htmlFor="name">Full Name</Label>
      <Input
        id="name"
        value={data.name || ''}
        onChange={(e) => updateData({ name: e.target.value })}
        placeholder="Enter your full name"
      />
    </div>
    <div>
      <Label htmlFor="email">Email</Label>
      <Input
        id="email"
        type="email"
        value={data.email || ''}
        onChange={(e) => updateData({ email: e.target.value })}
        placeholder="Enter your email"
      />
    </div>
  </div>
);

const ProjectInfoStep = ({ data, updateData }: { data: any; updateData: (data: any) => void }) => (
  <div className="space-y-4">
    <div>
      <Label htmlFor="projectName">Project Name</Label>
      <Input
        id="projectName"
        value={data.projectName || ''}
        onChange={(e) => updateData({ projectName: e.target.value })}
        placeholder="Enter project name"
      />
    </div>
    <div>
      <Label htmlFor="projectType">Project Type</Label>
      <Select value={data.projectType || ''} onValueChange={(value) => updateData({ projectType: value })}>
        <SelectTrigger>
          <SelectValue placeholder="Select project type" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="web">Web Application</SelectItem>
          <SelectItem value="mobile">Mobile App</SelectItem>
          <SelectItem value="desktop">Desktop App</SelectItem>
          <SelectItem value="api">API Service</SelectItem>
        </SelectContent>
      </Select>
    </div>
    <div>
      <Label htmlFor="description">Description</Label>
      <Textarea
        id="description"
        value={data.description || ''}
        onChange={(e) => updateData({ description: e.target.value })}
        placeholder="Describe your project"
        rows={4}
      />
    </div>
  </div>
);

const ReviewStep = ({ data }: { data: any }) => (
  <Card>
    <CardHeader>
      <CardTitle>Review Your Information</CardTitle>
    </CardHeader>
    <CardContent className="space-y-4">
      <div>
        <h4 className="font-medium">Personal Information</h4>
        <p className="text-sm text-muted-foreground">Name: {data.name || 'Not provided'}</p>
        <p className="text-sm text-muted-foreground">Email: {data.email || 'Not provided'}</p>
      </div>
      <div>
        <h4 className="font-medium">Project Information</h4>
        <p className="text-sm text-muted-foreground">Project: {data.projectName || 'Not provided'}</p>
        <p className="text-sm text-muted-foreground">Type: {data.projectType || 'Not provided'}</p>
        <p className="text-sm text-muted-foreground">Description: {data.description || 'Not provided'}</p>
      </div>
    </CardContent>
  </Card>
);

export const WizardExample: React.FC = () => {
  const { data, updateData, resetData } = useWizardData({
    name: '',
    email: '',
    projectName: '',
    projectType: '',
    description: ''
  });

  const steps: WizardStep[] = [
    {
      id: 'personal',
      title: 'Personal Information',
      description: 'Tell us about yourself',
      content: <PersonalInfoStep data={data} updateData={updateData} />,
      isValid: data.name && data.email
    },
    {
      id: 'project',
      title: 'Project Details',
      description: 'Describe your project',
      content: <ProjectInfoStep data={data} updateData={updateData} />,
      isValid: data.projectName && data.projectType
    },
    {
      id: 'review',
      title: 'Review & Complete',
      description: 'Review your information before submitting',
      content: <ReviewStep data={data} />,
      isValid: true
    }
  ];

  const handleComplete = (finalData: any) => {
    console.log('Wizard completed with data:', finalData);
    alert('Wizard completed! Check the console for the data.');
  };

  const handleStepChange = (stepIndex: number, step: WizardStep) => {
    console.log(`Changed to step ${stepIndex}: ${step.title}`);
  };

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-3xl font-bold mb-2">Wizard Component Example</h1>
        <p className="text-muted-foreground">
          This example demonstrates the Wizard component with forward and back navigation.
        </p>
        <button
          onClick={resetData}
          className="mt-4 px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80"
        >
          Reset Wizard
        </button>
      </div>

      <Wizard
        steps={steps}
        onComplete={handleComplete}
        onStepChange={handleStepChange}
        showProgress={true}
        allowSkip={false}
        data={data}
        onDataChange={updateData}
      />
    </div>
  );
};

export default WizardExample;
