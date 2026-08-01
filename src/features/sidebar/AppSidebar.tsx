import { useState } from "react";
import clsx from "clsx";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { useServersStore, type ServerStatus } from "@/store/serversStore";
import { NewServerDialog } from "./NewServerDialog";

const DOT: Record<ServerStatus, string> = {
  running: "bg-emerald-400",
  stopped: "bg-muted-foreground/50",
  starting: "bg-amber-400",
  stopping: "bg-amber-400",
  error: "bg-red-400",
};

export function AppSidebar() {
  const servers = useServersStore((s) => s.servers);
  const selectedServerId = useServersStore((s) => s.selectedServerId);
  const selectServer = useServersStore((s) => s.selectServer);
  const [dialogOpen, setDialogOpen] = useState(false);

  return (
    <Sidebar>
      <SidebarHeader>
        <h1 className="px-2 py-1 text-lg font-bold">
          Minecraft Server Launcher
        </h1>
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>
            <div className="flex w-full items-center justify-between">
              <span>Servers</span>
              <NewServerDialog open={dialogOpen} onOpenChange={setDialogOpen} />
            </div>
          </SidebarGroupLabel>
          <SidebarMenu>
            {servers.map((server) => (
              <SidebarMenuItem key={server.id}>
                <SidebarMenuButton
                  isActive={selectedServerId === server.id}
                  onClick={() => selectServer(server.id)}
                >
                  <span
                    className={clsx(
                      "size-1.5 shrink-0 rounded-none",
                      DOT[server.status],
                    )}
                  />
                  <span className="flex-1 truncate">{server.name}</span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            ))}
            {servers.length === 0 && (
              <p className="px-3 py-2 text-xs text-muted-foreground">
                No servers yet. Click + to add one.
              </p>
            )}
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter />
    </Sidebar>
  );
}
