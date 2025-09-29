import React, { useState, ReactNode } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { cn } from '@/lib/utils';

export interface WizardStep {
  id: string;
  title: string;
  content: ReactNode;
}

export interface WizardProps {
  steps: WizardStep[];
  onComplete?: (data: any) => void;
  className?: string;
  data?: any;
  onDataChange?: (data: any) => void;
}

export const Wizard: React.FC<WizardProps> = ({
  steps,
  onComplete,
  className,
  data = {},
  onDataChange
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [wizardData, setWizardData] = useState(data);

  const currentStepData = steps[currentStep];
  const isFirstStep = currentStep === 0;
  const isLastStep = currentStep === steps.length - 1;

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
    }
  };

  const goToPrevious = () => {
    if (!isFirstStep) {
      const prevStep = currentStep - 1;
      setCurrentStep(prevStep);
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
            </div>
            <div className="text-sm text-muted-foreground">
              Step {currentStep + 1} of {steps.length}
            </div>
          </div>
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
                onClick={goToPrevious}
                disabled={isFirstStep}
                className="flex items-center gap-2"
              >
                <ChevronLeft className="h-4 w-4" />
                Back
              </Button>
            </div>

            <div className="flex items-center gap-2">
              <Button
                onClick={goToNext}
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

export default Wizard;
