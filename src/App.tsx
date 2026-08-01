import { useEffect } from "react";
import "./App.css";
import { AppSidebar } from "./features/sidebar/AppSidebar";
import { SidebarInset, SidebarTrigger } from "./components/ui/sidebar";
import ServerPanel from "./features/serverPanel/ServerPanel";
import { useServersStore } from "./store/serversStore";
import { StatusBadge } from "./features/serverPanel/StatusBadge";
import { Button } from "./components/ui/button";
import { revealItemInDir } from "@tauri-apps/plugin-opener";

function App() {
  const init = useServersStore((s) => s.init);

  useEffect(() => {
    init();
  }, [init]);
  const servers = useServersStore((s) => s.servers);
  const selectedServerId = useServersStore((s) => s.selectedServerId);
  const server = servers.find((s) => s.id === selectedServerId);

  return (
    <>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-12 min-w-0 w-full items-center gap-2 border-b px-4">
          <SidebarTrigger />
          {server && (
            <div className="flex flex-row justify-between w-full">
              <div className="flex flex-row gap-2">
                <h2 className="text-lg font-semibold">{server.name}</h2>
                <StatusBadge status={server.status} />
              </div>
              <Button
                onClick={async () => {
                  await revealItemInDir(server.path);
                }}
              >
                Open folder
              </Button>
            </div>
          )}
        </header>
        <main className="flex min-w-0 flex-1 flex-col p-4">
          <ServerPanel />
        </main>
      </SidebarInset>
    </>
  );
}

export default App;
