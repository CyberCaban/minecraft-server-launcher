import { create } from "zustand";

interface Server {
  id: string;
  name: string;
  status: "running" | "stopped" | "starting" | "stopping";
}

interface ServersStoreState {
  servers: Server[];
  selectedServerId?: string;
  selectServer: (serverId: string) => void;
  addServer: (server: Server) => void;
}

const useServersStore = create<ServersStoreState>((set) => ({
  servers: [
    { id: "1", name: "Server 1", status: "stopped" },
    { id: "2", name: "Server 2", status: "stopped" },
    { id: "3", name: "Server 3", status: "stopped" },
  ],
  addServer: (server) =>
    set((state) => ({ servers: [...state.servers, server] })),
  selectServer: (serverId) => set({ selectedServerId: serverId }),
}));

export { useServersStore };
