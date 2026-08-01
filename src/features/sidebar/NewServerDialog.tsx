import { useRef, useState } from "react";
import { Loader2, Plus } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { useServersStore } from "@/store/serversStore";

type Mode = "template" | "yaml";

export function NewServerDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const createServer = useServersStore((s) => s.createServer);
  const selectServer = useServersStore((s) => s.selectServer);
  const templateName = useServersStore((s) => s.templateName);

  const [mode, setMode] = useState<Mode>("template");
  const [name, setName] = useState("");
  const [port, setPort] = useState("25565");
  const [memory, setMemory] = useState("2");
  const [yaml, setYaml] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  const reset = () => {
    setName("");
    setPort("25565");
    setMemory("2");
    setYaml("");
    setError(null);
  };

  const onFile = (file?: File | null) => {
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      setYaml(String(reader.result ?? ""));
      setMode("yaml");
    };
    reader.readAsText(file);
  };

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      if (!name.trim()) throw new Error("Enter a server name.");
      const source =
        mode === "template"
          ? { type: "template" as const, port: Number(port) || 25565, memoryGb: Number(memory) || 2 }
          : { type: "yaml" as const, content: yaml };
      if (mode === "yaml" && !yaml.trim()) {
        throw new Error("Paste a compose file or choose one.");
      }
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
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetTrigger
        render={
          <Button variant="ghost" size="icon-sm" className="h-7 w-7">
            <Plus />
          </Button>
        }
      />
      <SheetContent side="right" className="w-full sm:max-w-md">
        <SheetHeader>
          <SheetTitle>Add Server</SheetTitle>
          <SheetDescription>
            Create a new Minecraft server from a compose file.
          </SheetDescription>
        </SheetHeader>
        <div className="flex flex-col gap-4 px-8 pb-8">
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

          <div className="flex gap-2">
            {(["template", "yaml"] as const).map((m) => (
              <Button
                key={m}
                size="sm"
                variant={mode === m ? "default" : "outline"}
                onClick={() => setMode(m)}
                className="flex-1"
              >
                {m === "template" ? "Template" : "Custom YAML"}
              </Button>
            ))}
          </div>

          {mode === "template" ? (
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
          ) : (
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
          )}

          {error && <p className="text-xs text-destructive">{error}</p>}

          <Button onClick={submit} disabled={busy} className="mt-2">
            {busy ? <Loader2 className="animate-spin" /> : <Plus />}
            Create Server
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  );
}
