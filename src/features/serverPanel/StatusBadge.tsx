import clsx from "clsx";

import type { ServerStatus } from "@/store/serversStore";

const STYLES: Record<ServerStatus, string> = {
  running: "border-emerald-500/40 bg-emerald-500/10 text-emerald-500",
  stopped: "border-border bg-muted text-muted-foreground",
  starting: "border-amber-500/40 bg-amber-500/10 text-amber-500",
  stopping: "border-amber-500/40 bg-amber-500/10 text-amber-500",
  error: "border-red-500/40 bg-red-500/10 text-red-500",
};

const LABELS: Record<ServerStatus, string> = {
  running: "Running",
  stopped: "Stopped",
  starting: "Starting",
  stopping: "Stopping",
  error: "Error",
};

export function StatusBadge({ status }: { status: ServerStatus }) {
  return (
    <span
      className={clsx(
        "inline-flex items-center gap-1.5 border px-2 py-0.5 text-xs font-semibold uppercase tracking-wider",
        STYLES[status],
      )}
    >
      <span className="size-1.5 bg-current" />
      {LABELS[status]}
    </span>
  );
}
