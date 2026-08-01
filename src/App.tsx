import { useEffect } from "react";
import "./App.css";
import { AppSidebar } from "./features/sidebar/AppSidebar";
import { SidebarTrigger } from "./components/ui/sidebar";
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
      <main className="container">
        <SidebarTrigger />
        <ServerPanel />
      </main>
    </>
  );
}

export default App;
