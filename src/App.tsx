import "./App.css";
import { AppSidebar } from "./features/sidebar/AppSidebar";
import { SidebarTrigger } from "./components/ui/sidebar";
import ServerPanel from "./features/serverPanel/ServerPanel";

function App() {
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
