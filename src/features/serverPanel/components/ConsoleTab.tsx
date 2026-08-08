import { AlertCircle, Play, RotateCw, Square, Trash2, X } from "lucide-react";

import { Button } from "@/components/ui/button";
import { useServersStore, type DockerStatus } from "@/store/serversStore";
import { Console } from "./Console";
import { confirm } from "@tauri-apps/plugin-dialog";

function DockerWarning({ docker }: { docker: DockerStatus }) {
  const updateDockerStatus = useServersStore((s) => s.updateDockerStatus);
  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-lg font-semibold">Docker unavailable</h2>
      <p className="text-sm text-muted-foreground">
        {docker.error ?? "Docker engine is not running."} Make sure Docker
        Desktop is installed and running <br /> <br />
        <Button size={"sm"} variant={"secondary"} onClick={updateDockerStatus}>
          <RotateCw className="" />
          Refresh
        </Button>
      </p>
    </div>
  );
}

function ErrorBanner({ message }: { message: string }) {
  const clearError = useServersStore((s) => s.clearError);
  return (
    <div
      role="alert"
      className="flex items-start justify-between gap-3 rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm text-destructive"
    >
      <div className="flex items-start gap-2">
        <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
        <span className="break-all font-mono">{message}</span>
      </div>
      <button
        onClick={clearError}
        aria-label="Dismiss"
        className="shrink-0 rounded p-0.5 opacity-70 transition-opacity hover:opacity-100"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

export default function ConsoleTab() {
  const servers = useServersStore((s) => s.servers);
  const selectedServerId = useServersStore((s) => s.selectedServerId);
  const docker = useServersStore((s) => s.docker);
  const lastError = useServersStore((s) => s.lastError);
  const startServer = useServersStore((s) => s.startServer);
  const stopServer = useServersStore((s) => s.stopServer);
  const restartServer = useServersStore((s) => s.restartServer);
  const removeServer = useServersStore((s) => s.removeServer);

  const server = servers.find((s) => s.id === selectedServerId);

  if (docker && !docker.engineOk) {
    return <DockerWarning docker={docker} />;
  }

  if (!server) {
    return (
      <div className="flex flex-col gap-4">
        <h2 className="text-lg font-semibold">Server Panel</h2>
        <p className="text-sm text-muted-foreground">
          Select a server from the sidebar or create a new one to get started.
        </p>
      </div>
    );
  }

  const running = server.status === "running";
  const busy = server.status === "starting" || server.status === "stopping";

  const handleRemove = async () => {
    if (
      await confirm(
        `Delete server "${server.name}"? This stops its containers and removes its folder.`,
        {
          title: "Confirm Delete",
          kind: "warning",
          okLabel: "Delete",
          cancelLabel: "Cancel",
        },
      )
    ) {
      try {
        await removeServer(server.id);
      } catch {
        // ignore
      }
    }
  };

  return (
    <div className="flex flex-col gap-4 text-left">
      {lastError && <ErrorBanner message={lastError} />}
      <div className="flex items-start justify-between gap-4">
        <div className="flex flex-wrap gap-2">
          {!running && (
            <Button
              size="sm"
              disabled={busy}
              onClick={() => startServer(server.id)}
            >
              <Play />
              Start
            </Button>
          )}
          {running && (
            <>
              <Button
                size="sm"
                variant="secondary"
                disabled={busy}
                onClick={() => stopServer(server.id)}
              >
                <Square />
                Stop
              </Button>
              <Button
                size="sm"
                variant="outline"
                disabled={busy}
                onClick={() => restartServer(server.id)}
              >
                <RotateCw />
                Restart
              </Button>
            </>
          )}
        </div>
        <Button
          size="sm"
          variant="destructive"
          disabled={busy}
          onClick={handleRemove}
        >
          <Trash2 />
          Remove
        </Button>
      </div>
      <Console serverId={server.id} />
    </div>
  );
}
