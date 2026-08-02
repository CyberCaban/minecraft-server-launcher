import { Button } from "@/components/ui/button";
import { FunctionComponent } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface ExistingSourceFormProps {
  composePath: string;
  setComposePath: (path: string) => void;
}

const ExistingSourceForm: FunctionComponent<ExistingSourceFormProps> = ({
  composePath,
  setComposePath,
}) => {
  const handleFileSelect = async () => {
    const file = await open({
      directory: false,
      multiple: false,
    });
    if (!file) return;
    setComposePath(file);
  };
  return (
    <div className="flex flex-col gap-2">
      <Button variant="outline" size="sm" onClick={() => handleFileSelect()}>
        Choose compose file…
      </Button>

      <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
        {composePath}
      </label>
    </div>
  );
};

export default ExistingSourceForm;
