import { Button } from "@/components/ui/button";
import { FunctionComponent, useRef } from "react";

interface YamlSourceFormProps {
  yaml: string;
  setYaml: (yaml: string) => void;
}

const YamlSourceForm: FunctionComponent<YamlSourceFormProps> = ({
  yaml,
  setYaml,
}) => {
  const fileRef = useRef<HTMLInputElement>(null);
  const onFile = (file?: File | null) => {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      setYaml(String(reader.result ?? ""));
    };
    reader.readAsText(file);
  };
  return (
    <div className="flex flex-col gap-2">
      <input
        ref={fileRef}
        type="file"
        accept=".yml,.yaml"
        className="hidden"
        onChange={(e) => onFile(e.target.files?.[0])}
      />
      <Button
        variant="outline"
        size="sm"
        onClick={() => fileRef.current?.click()}
      >
        Choose compose file…
      </Button>
      <textarea
        value={yaml}
        onChange={(e) => setYaml(e.target.value)}
        rows={16}
        placeholder="Paste docker-compose.yml content here"
        spellCheck={false}
        className="w-full resize-y border border-border bg-transparent p-3 font-mono text-xs outline-none focus-visible:border-ring"
      />
    </div>
  );
};

export default YamlSourceForm;
