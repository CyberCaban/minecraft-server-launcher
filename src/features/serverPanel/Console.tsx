import { useEffect, useRef, useState } from "react";
import { SendHorizontal } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useServersStore } from "@/store/serversStore";

export function Console({ serverId }: { serverId: string }) {
  const servers = useServersStore((s) => s.servers);
  const consoleLines = useServersStore((s) => s.consoleLines);
  const sendCommand = useServersStore((s) => s.sendCommand);
  const loadConsole = useServersStore((s) => s.loadConsole);
  const clearConsole = useServersStore((s) => s.clearConsole);

  const server = servers.find((s) => s.id === serverId);
  const lines = consoleLines[serverId] ?? [];
  const [command, setCommand] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);
  const stickToBottom = useRef(true);

  useEffect(() => {
    loadConsole(serverId);
  }, [serverId, loadConsole]);

  useEffect(() => {
    const el = scrollRef.current;
    if (el && stickToBottom.current) {
      el.scrollTop = el.scrollHeight;
    }
  }, [lines.length]);

  const handleScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    stickToBottom.current = el.scrollHeight - el.scrollTop - el.clientHeight < 60;
  };

  const submit = async () => {
    const cmd = command.trim();
    if (!cmd) return;
    setCommand("");
    try {
      await sendCommand(serverId, cmd);
    } catch {
      // rcon failure surfaces via logs; ignore
    }
  };

  const running = server?.status === "running";
  const canSend = running && Boolean(server?.hasRcon);

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Console
        </p>
        <button
          type="button"
          onClick={() => clearConsole(serverId)}
          className="text-xs text-muted-foreground transition-colors hover:text-foreground"
        >
          Clear
        </button>
      </div>
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="h-96 overflow-y-auto border border-border bg-black/40 p-3 font-mono text-xs leading-relaxed"
      >
        {lines.length === 0 ? (
          <p className="text-muted-foreground">
            No console output yet. Start the server to see logs.
          </p>
        ) : (
          lines.map((l, i) => (
            <div key={`${l.ts}-${i}`} className="whitespace-pre-wrap break-words">
              {l.line}
            </div>
          ))
        )}
      </div>
      <div className="flex items-center gap-2">
        <Input
          value={command}
          onChange={(e) => setCommand(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
          placeholder={
            canSend
              ? "Enter a server command…"
              : running
                ? "Console input disabled (RCON off)"
                : "Start the server to send commands"
          }
          disabled={!canSend}
          className="flex-1"
        />
        <Button onClick={submit} disabled={!canSend} size="sm" aria-label="Send">
          <SendHorizontal />
        </Button>
      </div>
      {running && !server?.hasRcon && (
        <p className="text-xs text-muted-foreground">
          RCON is disabled in this compose file, so console commands are
          unavailable.
        </p>
      )}
    </div>
  );
}
