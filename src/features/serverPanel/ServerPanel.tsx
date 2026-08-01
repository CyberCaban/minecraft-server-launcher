import { useServersStore } from "@/store/serversStore";
export default function ServerPanel() {
//   const { selectedServerId, servers } = useServersStore();
  return (
    <div className="flex flex-col gap-4">
      <h2 className="text-lg font-semibold">Server Panel</h2>
      <p className="text-sm text-muted-foreground">
        This is the server panel where you can manage your Minecraft servers.
      </p>
    </div>
  );
}
