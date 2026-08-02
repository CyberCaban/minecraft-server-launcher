import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type ServerStatus =
  | "running"
  | "stopped"
  | "starting"
  | "stopping"
  | "error";

export interface Server {
  id: string;
  name: string;
  project: string;
  path: string;
  status: ServerStatus;
  hasRcon: boolean;
}

export interface DockerStatus {
  engineOk: boolean;
  composeOk: boolean;
  error: string | null;
}

export type CreateServerSource = "template" | "yaml" | "existing";
export type TemplateCreateSource = {
  port: number;
  memoryGb: number;
};
export type ExistingCreateSource = {
  composePath: string;
};
export type YamlCreateSource = { content: string };
export type CreateServerPayload = {
  type: CreateServerSource;
} & (TemplateCreateSource | YamlCreateSource | ExistingCreateSource);

interface ServerLogLine {
  line: string;
  ts: number;
}

interface ServersStoreState {
  servers: Server[];
  selectedServerId?: string;
  docker: DockerStatus | null;
  templateName: string;
  consoleLines: Record<string, ServerLogLine[]>;
  lastError: string | null;
  initialized: boolean;
  init: () => Promise<void>;
  selectServer: (serverId: string) => void;
  createServer: (name: string, source: CreateServerPayload) => Promise<Server>;
  startServer: (serverId: string) => Promise<void>;
  stopServer: (serverId: string) => Promise<void>;
  restartServer: (serverId: string) => Promise<void>;
  removeServer: (serverId: string) => Promise<void>;
  sendCommand: (serverId: string, command: string) => Promise<string>;
  loadConsole: (serverId: string) => Promise<void>;
  clearConsole: (serverId: string) => void;
  clearError: () => void;
}

let initPromise: Promise<void> | null = null;

const useServersStore = create<ServersStoreState>((set) => ({
  servers: [],
  selectedServerId: undefined,
  docker: null,
  templateName: "Forge 1.20.1",
  consoleLines: {},
  lastError: null,
  initialized: false,

  init: async () => {
    if (initPromise) return initPromise;
    initPromise = (async () => {
      const unlisteners: UnlistenFn[] = [];
      unlisteners.push(
        await listen<{ serverId: string; line: string }>(
          "server-log",
          (event) => {
            const { serverId, line } = event.payload;
            set((state) => {
              const current = state.consoleLines[serverId] ?? [];
              const next = [...current, { line, ts: Date.now() }].slice(-1000);
              return {
                consoleLines: { ...state.consoleLines, [serverId]: next },
              };
            });
          },
        ),
      );
      unlisteners.push(
        await listen<{ serverId: string; status: ServerStatus }>(
          "server-status",
          (event) => {
            const { serverId, status } = event.payload;
            set((state) => ({
              servers: state.servers.map((s) =>
                s.id === serverId ? { ...s, status } : s,
              ),
            }));
          },
        ),
      );

      const docker = await invoke<DockerStatus>("docker_status");
      const templateName = await invoke<string>("get_template_info");
      const servers = await invoke<Server[]>("list_servers");
      set({ docker, templateName, servers, initialized: true });
      await invoke("refresh_status");
      window.addEventListener("beforeunload", () => {
        unlisteners.forEach((u) => u());
      });
    })();
    return initPromise;
  },

  selectServer: (serverId) => set({ selectedServerId: serverId }),

  createServer: async (name, source) => {
    try {
      const server = await invoke<Server>("create_server", { name, source });
      set((state) => ({ servers: [...state.servers, server] }));
      return server;
    } catch (e) {
      set({ lastError: String(e) });
      throw e;
    }
  },

  startServer: async (serverId) => {
    try {
      await invoke("start_server", { serverId });
    } catch (e) {
      set({ lastError: String(e) });
      throw e;
    }
  },

  stopServer: async (serverId) => {
    try {
      await invoke("stop_server", { serverId });
    } catch (e) {
      set({ lastError: String(e) });
      throw e;
    }
  },

  restartServer: async (serverId) => {
    try {
      await invoke("restart_server", { serverId });
    } catch (e) {
      set({ lastError: String(e) });
      throw e;
    }
  },

  removeServer: async (serverId) => {
    try {
      await invoke("remove_server", { serverId });
      set((state) => {
        const servers = state.servers.filter((s) => s.id !== serverId);
        const consoleLines = { ...state.consoleLines };
        delete consoleLines[serverId];
        const selectedServerId =
          state.selectedServerId === serverId
            ? servers[0]?.id
            : state.selectedServerId;
        return { servers, consoleLines, selectedServerId };
      });
    } catch (e) {
      set({ lastError: String(e) });
      throw e;
    }
  },

  sendCommand: async (serverId, command) => {
    try {
      return await invoke<string>("send_command", { serverId, command });
    } catch (e) {
      set({ lastError: String(e) });
      throw e;
    }
  },

  loadConsole: async (serverId) => {
    try {
      const lines = await invoke<string[]>("get_server_logs", {
        serverId,
        lines: 300,
      });
      set((state) => ({
        consoleLines: {
          ...state.consoleLines,
          [serverId]: lines.map((line) => ({ line, ts: Date.now() })),
        },
      }));
    } catch {
      // server not running; ignore
    }
  },

  clearConsole: (serverId) => {
    set((state) => ({
      consoleLines: { ...state.consoleLines, [serverId]: [] },
    }));
  },

  clearError: () => set({ lastError: null }),
}));

export { useServersStore };
