import { Input } from "@/components/ui/input";
import { useServersStore } from "@/store/serversStore";
import { FunctionComponent } from "react";

interface TemplateSourceProps {
  port: string;
  setPort: (port: string) => void;

  memory: string;
  setMemory: (port: string) => void;
}

const TemplateSource: FunctionComponent<TemplateSourceProps> = ({
  port,
  setPort,
  memory,
  setMemory,
}) => {
  const templateName = useServersStore((s) => s.templateName);
  return (
    <>
      <p className="text-xs text-muted-foreground">
        Uses the {templateName} image with RCON enabled.
      </p>
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Port
          </label>
          <Input
            value={port}
            onChange={(e) => setPort(e.target.value)}
            type="number"
          />
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Memory (GB)
          </label>
          <Input
            value={memory}
            onChange={(e) => setMemory(e.target.value)}
            type="number"
          />
        </div>
      </div>
    </>
  );
};

export default TemplateSource;
