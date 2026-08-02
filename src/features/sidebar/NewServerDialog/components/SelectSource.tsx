import { Button } from "@/components/ui/button";
import { CreateServerSource } from "@/store/serversStore";
import { FunctionComponent } from "react";

interface SelectSourceProps {
  mode: CreateServerSource;
  setMode: (mode: CreateServerSource) => void;
}

const modeMap: Record<CreateServerSource, string> = {
  yaml: "Custom YAML",
  template: "Template",
  existing: "Add existing server",
};

const SelectSource: FunctionComponent<SelectSourceProps> = ({
  mode,
  setMode,
}) => {
  return (
    <div className="flex flex-wrap gap-2">
      {(["template", "yaml", "existing"] as Array<CreateServerSource>).map(
        (m) => (
          <Button
            key={m}
            size="sm"
            variant={mode === m ? "default" : "outline"}
            onClick={() => setMode(m)}
            className="flex-1"
          >
            {modeMap[m]}
          </Button>
        ),
      )}
    </div>
  );
};

export default SelectSource;
