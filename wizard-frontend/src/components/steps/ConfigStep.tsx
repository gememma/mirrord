import { useState } from "react";

const ConfigStep = () => {
    const [currentTab, setCurrentTab] = useState<string>("target");

    return {
    id: "config-new-user",
    title: "Configuration Setup",
    content: (
      <div className="space-y-6">
        <div className="text-center space-y-4 pt-4">
          <p className="text-sm text-muted-foreground">
            Configure your mirrord settings using the tabs below
            {/* TODO: tabs component here */}
          </p>
        </div>
      </div>
    )
  };
};

export default ConfigStep