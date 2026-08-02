import { useState } from "react";
import { Box, Plus, Server } from "lucide-react";
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
import { TooltipProvider } from "@/components/ui/tooltip";
import { useServersStore, type ServerStatus } from "@/store/serversStore";
import { NewServerDialog } from "./NewServerDialog/NewServerDialog";
import { ThemeToggle } from "@/components/themeToggle";

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
    <TooltipProvider>
      <Sidebar collapsible="icon">
        <SidebarHeader>
          <div className="flex flex-row items-center justify-between gap-2 px-2 py-1.5">
            <div className="flex flex-row items-center gap-2">
              <Box className="size-5 shrink-0" />
              <span className="truncate text-sm font-bold group-data-[collapsible=icon]:hidden">
                Server Launcher
              </span>
            </div>
            <ThemeToggle className="group-data-[collapsible=icon]:hidden" />
          </div>
        </SidebarHeader>
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupLabel>Servers</SidebarGroupLabel>
            <SidebarMenu>
              {servers.map((server) => (
                <SidebarMenuItem key={server.id}>
                  <SidebarMenuButton
                    isActive={selectedServerId === server.id}
                    onClick={() => selectServer(server.id)}
                    tooltip={server.name}
                  >
                    <Server className="size-4 shrink-0" />
                    <span className="min-w-0 flex-1 truncate">
                      {server.name}
                    </span>
                    <span
                      className={clsx(
                        "size-2 shrink-0 rounded-full",
                        DOT[server.status],
                      )}
                    />
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
              {servers.length === 0 && (
                <p className="px-3 py-2 text-xs text-muted-foreground group-data-[collapsible=icon]:hidden">
                  No servers yet. Click + to add one.
                </p>
              )}
            </SidebarMenu>
          </SidebarGroup>
        </SidebarContent>
        <SidebarFooter>
          <SidebarMenu>
            <SidebarMenuItem>
              <NewServerDialog
                open={dialogOpen}
                onOpenChange={setDialogOpen}
                trigger={
                  <SidebarMenuButton tooltip="Add server" className="w-full">
                    <Plus className="size-4 shrink-0" />
                    <span className="flex-1 text-left">New server</span>
                  </SidebarMenuButton>
                }
              />
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarFooter>
      </Sidebar>
    </TooltipProvider>
  );
}
