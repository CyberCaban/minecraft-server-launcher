import { useEffect } from "react";
import "./App.css";
import { AppSidebar } from "./features/sidebar/AppSidebar";
import { SidebarInset, SidebarTrigger } from "./components/ui/sidebar";
import ServerPanel from "./features/serverPanel/ServerPanel";
import { useServersStore } from "./store/serversStore";

function App() {
  const init = useServersStore((s) => s.init);

  useEffect(() => {
    init();
  }, [init]);

  return (
    <>
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-12 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger />
        </header>
        <main className="flex min-w-0 flex-1 flex-col p-4">
          <ServerPanel />
        </main>
      </SidebarInset>
    </>
  );
}

export default App;
