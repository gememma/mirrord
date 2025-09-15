import { Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

interface OverviewSectionProps {
  onWizardOpen: (mode: 'create' | 'overview') => void;
}

export const OverviewSection = ({ onWizardOpen }: OverviewSectionProps) => {
  return (
    <div className="p-6">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">mirrord Overview</h1>
          <p className="text-muted-foreground">
            Learn how mirrord works and explore its capabilities
          </p>
        </div>
        
        <div className="grid gap-6">
          <Card className="bg-gradient-card border-border/50">
            <CardHeader>
              <CardTitle>What is mirrord?</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground">
                mirrord lets you run your local code in the context of your cloud environment. Unlike traditional development tools, you can test your code with real cloud dependencies, data, and configuration without deploying anything.
              </p>
            </CardContent>
          </Card>

          <div className="grid md:grid-cols-3 gap-4">
            <Card className="bg-gradient-card border-border/50">
              <CardHeader>
                <CardTitle className="text-lg">Filtering Mode</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  Your local service only handles matching traffic based on HTTP Headers/Path. This allows you to control what flows in the system you want to affect.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-gradient-card border-border/50">
              <CardHeader>
                <CardTitle className="text-lg">Mirror Mode</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  Your local service receives a copy of the incoming traffic. Best for debugging existing traffic without disrupting the environment.
                </p>
              </CardContent>
            </Card>

            <Card className="bg-gradient-card border-border/50">
              <CardHeader>
                <CardTitle className="text-lg">Replace Mode</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  Completely substitute the remote service. Useful when you want to work against queues and have full control.
                </p>
              </CardContent>
            </Card>
          </div>

          <div className="flex justify-center mt-8">
            <Button onClick={() => onWizardOpen('create')} className="bg-gradient-primary hover:shadow-glow">
              <Plus className="h-4 w-4 mr-2" />
              Create Your Configuration
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};
