import React, { useState, ReactNode } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { cn } from '@/lib/utils';

export interface WizardStep {
  id: string;
  title: string;
  description?: string;
  content: ReactNode;
  isValid?: boolean;
  isOptional?: boolean;
}

export interface WizardProps {
  steps: WizardStep[];
  onComplete?: (data: any) => void;
  onStepChange?: (stepIndex: number, step: WizardStep) => void;
  className?: string;
  showProgress?: boolean;
  allowSkip?: boolean;
  initialStep?: number;
  data?: any;
  onDataChange?: (data: any) => void;
}

export const Wizard: React.FC<WizardProps> = ({
  steps,
  onComplete,
  onStepChange,
  className,
  showProgress = true,
  allowSkip = false,
  initialStep = 0,
  data = {},
  onDataChange
}) => {
  const [currentStep, setCurrentStep] = useState(initialStep);
  const [wizardData, setWizardData] = useState(data);

  const currentStepData = steps[currentStep];
  const isFirstStep = currentStep === 0;
  const isLastStep = currentStep === steps.length - 1;
  const progress = ((currentStep + 1) / steps.length) * 100;

  const updateData = (newData: any) => {
    const updatedData = { ...wizardData, ...newData };
    setWizardData(updatedData);
    onDataChange?.(updatedData);
  };

  const goToNext = () => {
    if (isLastStep) {
      onComplete?.(wizardData);
    } else {
      const nextStep = currentStep + 1;
      setCurrentStep(nextStep);
      onStepChange?.(nextStep, steps[nextStep]);
    }
  };

  const goToPrevious = () => {
    if (!isFirstStep) {
      const prevStep = currentStep - 1;
      setCurrentStep(prevStep);
      onStepChange?.(prevStep, steps[prevStep]);
    }
  };

  const goToStep = (stepIndex: number) => {
    if (stepIndex >= 0 && stepIndex < steps.length) {
      setCurrentStep(stepIndex);
      onStepChange?.(stepIndex, steps[stepIndex]);
    }
  };

  const canProceed = () => {
    const step = steps[currentStep];
    return step.isOptional || step.isValid !== false;
  };

  const handleNext = () => {
    if (canProceed()) {
      goToNext();
    }
  };

  const handlePrevious = () => {
    goToPrevious();
  };

  const handleSkip = () => {
    if (allowSkip && !isLastStep) {
      goToNext();
    }
  };

  return (
    <div className={cn("w-full max-w-4xl mx-auto", className)}>
      <Card className="w-full">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-2xl font-bold">
                {currentStepData.title}
              </CardTitle>
              {currentStepData.description && (
                <p className="text-muted-foreground mt-2">
                  {currentStepData.description}
                </p>
              )}
            </div>
            <div className="text-sm text-muted-foreground">
              Step {currentStep + 1} of {steps.length}
            </div>
          </div>
          
          {showProgress && (
            <div className="mt-4">
              <div className="flex items-center justify-between text-sm text-muted-foreground mb-2">
                <span>Progress</span>
                <span>{Math.round(progress)}%</span>
              </div>
              <Progress value={progress} className="h-2" />
            </div>
          )}
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Step Content */}
          <div className="min-h-[400px]">
            {React.cloneElement(currentStepData.content as React.ReactElement, {
              data: wizardData,
              updateData,
              currentStep,
              totalSteps: steps.length
            })}
          </div>

          {/* Navigation */}
          <div className="flex items-center justify-between pt-6 border-t">
            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                onClick={handlePrevious}
                disabled={isFirstStep}
                className="flex items-center gap-2"
              >
                <ChevronLeft className="h-4 w-4" />
                Back
              </Button>
              
              {allowSkip && !isLastStep && (
                <Button
                  variant="ghost"
                  onClick={handleSkip}
                  className="text-muted-foreground"
                >
                  Skip
                </Button>
              )}
            </div>

            <div className="flex items-center gap-2">
              {/* Step Indicators */}
              <div className="flex items-center gap-1">
                {steps.map((_, index) => (
                  <button
                    key={index}
                    onClick={() => goToStep(index)}
                    className={cn(
                      "w-2 h-2 rounded-full transition-colors",
                      index === currentStep
                        ? "bg-primary"
                        : index < currentStep
                        ? "bg-primary/50"
                        : "bg-muted"
                    )}
                    disabled={index > currentStep}
                  />
                ))}
              </div>

              <Button
                onClick={handleNext}
                disabled={!canProceed()}
                className="flex items-center gap-2"
              >
                {isLastStep ? 'Complete' : 'Next'}
                {!isLastStep && <ChevronRight className="h-4 w-4" />}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

// Hook for using wizard data
export const useWizardData = (initialData: any = {}) => {
  const [data, setData] = useState(initialData);

  const updateData = (newData: any) => {
    setData(prev => ({ ...prev, ...newData }));
  };

  const resetData = () => {
    setData(initialData);
  };

  return { data, updateData, resetData };
};

// Higher-order component for wizard steps
export const withWizardStep = <P extends object>(
  Component: React.ComponentType<P & { data: any; updateData: (data: any) => void }>
) => {
  return (props: P & { data: any; updateData: (data: any) => void }) => {
    return <Component {...props} />;
  };
};

export default Wizard;
