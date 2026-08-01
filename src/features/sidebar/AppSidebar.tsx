import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { ChevronDown } from "lucide-react";
import { useServersStore } from "@/store/serversStore";
import { Button } from "@/components/ui/button";
import clsx from "clsx";

export function AppSidebar() {
  const { servers, selectedServerId, selectServer } = useServersStore();
  return (
    <Sidebar>
      <SidebarHeader>
        <h1 className="text-lg font-bold">Minecraft Server Launcher</h1>
      </SidebarHeader>
      <SidebarContent>
        <Collapsible className="group/collapsible">
          <SidebarGroup>
            <SidebarGroupLabel>
              <CollapsibleTrigger
                render={
                  <Button className="flex flex-row items-center gap-2 text-sm font-medium leading-none text-muted-foreground transition-colors hover:text-foreground data-[state=open]/collapsible:text-foreground">
                    Servers
                    <ChevronDown className="ml-auto transition-transform group-data-panel-open/button:rotate-180" />
                  </Button>
                }
              ></CollapsibleTrigger>
            </SidebarGroupLabel>
            <CollapsibleContent>
              <SidebarMenu>
                {servers.map((server) => (
                  <SidebarMenuItem
                    key={server.id}
                    onClick={() => selectServer(server.id)}
                    className={clsx({
                      "bg-muted text-muted-foreground":
                        selectedServerId === server.id,
                    })}
                  >
                    {server.name}
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </CollapsibleContent>
          </SidebarGroup>
        </Collapsible>
        <SidebarGroup />
      </SidebarContent>
      <SidebarFooter />
    </Sidebar>
  );
}
