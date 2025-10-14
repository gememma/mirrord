export const validateJson = (
  jsonString: string,
  setJsonError: (error: string) => void
): boolean => {
  try {
    JSON.parse(jsonString);
    setJsonError("");
    return true;
  } catch (error) {
    setJsonError(
      `Invalid JSON: ${
        error instanceof Error ? error.message : "Unknown error"
      }`
    );
    return false;
  }
};

export const updateConfigFromJson = (
  jsonString: string,
  setConfig: React.Dispatch<React.SetStateAction<any>>,
  setJsonError: (error: string) => void
): void => {
  if (validateJson(jsonString, setJsonError)) {
    try {
      const parsedConfig = JSON.parse(jsonString);
      setConfig((prevConfig) => ({
        ...prevConfig,
        target: parsedConfig.target || prevConfig.target,
        agent: {
          scaledown: parsedConfig.agent?.scaledown || false,
          copyTarget: parsedConfig.agent?.copy_target || false,
        },
      }));
    } catch (error) {
      console.error("Error updating config from JSON:", error);
    }
  }
};
