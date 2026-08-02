import { FunctionComponent, useState } from "react";
import TemplateSource from "./sourceForms/TemplateSource";
import YamlSourceForm from "./sourceForms/YamlSourceForm";
import { Button } from "@/components/ui/button";
import { Loader2, Plus } from "lucide-react";
import { Input } from "@/components/ui/input";
import { CreateServerSource, CreateServerPayload, useServersStore } from "@/store/serversStore";

interface CreateServerSourceProps {
  mode: CreateServerSource;
  onOpenChange: (value: boolean) => void;
}

const CreateServerSourceForm: FunctionComponent<CreateServerSourceProps> = ({
  mode,
  onOpenChange,
}) => {
  const createServer = useServersStore((s) => s.createServer);
  const selectServer = useServersStore((s) => s.selectServer);

  const [port, setPort] = useState("25565");
  const [memory, setMemory] = useState("2");
  const [yaml, setYaml] = useState("");
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const reset = () => {
    setName("");
    setPort("25565");
    setMemory("2");
    setYaml("");
    setError(null);
  };

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      if (!name.trim()) throw new Error("Enter a server name.");
      const source: CreateServerPayload =
        mode === "template"
          ? {
              type: "template" as const,
              port: Number(port) || 25565,
              memoryGb: Number(memory) || 2,
            }
          : { type: "yaml" as const, content: yaml };
      if (mode === "yaml" && !yaml.trim()) {
        throw new Error("Paste a compose file or choose one.");
      }
      console.log(source);

      const server = await createServer(name.trim(), source);
      selectServer(server.id);
      reset();
      onOpenChange(false);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <div className="flex flex-col gap-1">
        <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Name
        </label>
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My server"
        />
      </div>

      {mode === "template" && (
        <TemplateSource
          memory={memory}
          setMemory={setMemory}
          port={port}
          setPort={setPort}
        />
      )}

      {mode === "yaml" && <YamlSourceForm yaml={yaml} setYaml={setYaml} />}

      {error && <p className="text-xs text-destructive">{error}</p>}

      <Button onClick={submit} disabled={busy} className="mt-2">
        {busy ? <Loader2 className="animate-spin" /> : <Plus />}
        Create Server
      </Button>
    </>
  );
};

export default CreateServerSourceForm;
