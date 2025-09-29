import { useState, useEffect, ReactNode } from "react";
import { WizardStep } from "@/components/Wizard";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ArrowRight, BookOpen, Plus, Zap } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ConfigWizard } from "./ConfigWizard";
import { ConfigData } from "@/types/config";

interface PanelProps {
  title: ReactNode;
  content: string;
  buttonText: string;
  buttonColor: "purple" | "gray";
  steps: WizardStep[];
  isReturning: boolean;
}

const Panel = ({
  title,
  content,
  buttonText,
  buttonColor,
  steps,
  isReturning,
}: PanelProps) => {
  const [showWizard, setShowWizard] = useState(false);

  const handleWizardOpen = (steps: WizardStep[]) => {
    // TODO: take steps param and open correct wizard
    setShowWizard(true);
  };

  const cardButton = () => {
    if (buttonColor == "purple") {
      return (
        <Button
          onClick={() => handleWizardOpen(steps)}
          className="w-full bg-gradient-primary hover:shadow-glow"
        >
          {buttonText}
          <ArrowRight className="h-4 w-4 ml-2" />
        </Button>
      );
    } else {
      return (
        <Button
          variant="outline"
          onClick={() => handleWizardOpen(steps)}
          className="w-full"
        >
          {buttonText}
          <ArrowRight className="h-4 w-4 ml-2" />
        </Button>
      );
    }
  };

  return (
    <div>
      <div className="grid gap-6 sm:gap-8 max-w-4xl mx-auto">
        <ConfigWizard
          isOpen={showWizard}
          onClose={() => setShowWizard(false)}
          onSave={function (config: ConfigData): void {
            throw new Error("Function not implemented.");
          }}
          isReturning={isReturning}
        />
        <Card className="bg-gradient-card border-border/50 hover:shadow-glow transition-all duration-300">
          <CardHeader>
            {title}
            <CardDescription>{content}</CardDescription>
          </CardHeader>
          <CardContent>{cardButton()}</CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Panel;
