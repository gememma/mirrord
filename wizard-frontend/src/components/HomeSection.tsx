import { Plus, BookOpen, ArrowRight, FileCode2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

interface Service {
  name: string;
  configs: any[];
}

interface HomeSectionProps {
  services: Service[];
  onWizardOpen: (mode: 'create' | 'overview') => void;
  onSectionChange: (section: string) => void;
}

export const HomeSection = ({ services, onWizardOpen, onSectionChange }: HomeSectionProps) => {
  return (
    <div className="min-h-screen w-full bg-background flex items-center justify-center p-6">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-8 sm:mb-12">
          <h1 className="text-2xl sm:text-3xl font-bold mb-4 text-foreground">
            Welcome back! 👋
          </h1>
          <p className="text-muted-foreground text-sm sm:text-lg max-w-2xl mx-auto">
            Ready to create your next mirrord configuration or learn more about the platform?
          </p>
        </div>

        <div className="grid gap-6 sm:gap-8 md:grid-cols-2 max-w-4xl mx-auto">
          <Card className="bg-gradient-card border-border/50 hover:shadow-glow transition-all duration-300">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Plus className="h-5 w-5" />
                Create Configuration
              </CardTitle>
              <CardDescription>
                Use our wizard to create a working mirrord.json configuration file for your project
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button onClick={() => onWizardOpen('create')} className="w-full bg-gradient-primary hover:shadow-glow">
                Create New Config
                <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            </CardContent>
          </Card>

          <Card className="bg-gradient-card border-border/50 hover:shadow-glow transition-all duration-300">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <BookOpen className="h-5 w-5" />
                Learn About mirrord
              </CardTitle>
              <CardDescription>
                Explore how mirrord works and understand the different modes and configurations
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Button variant="outline" onClick={() => onWizardOpen('overview')} className="w-full">
                View Overview
                <ArrowRight className="h-4 w-4 ml-2" />
              </Button>
            </CardContent>
          </Card>
        </div>

        {services.length > 0 && (
          <div className="mt-12">
            <div className="text-center mb-6">
              <h2 className="text-xl font-semibold mb-2">Your Configurations</h2>
              <p className="text-muted-foreground text-sm">
                You have {services.length} service{services.length !== 1 ? 's' : ''} with configurations
              </p>
            </div>
            <div className="flex justify-center">
              <Button variant="outline" onClick={() => onSectionChange("configs")}>
                <FileCode2 className="h-4 w-4 mr-2" />
                Manage Config Files
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
