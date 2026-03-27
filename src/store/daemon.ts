import { create } from "zustand";
import { api } from "@/lib/api";

interface DaemonState {
  status: any | null;
  dashboard: any | null;
  cost: any | null;
  loading: boolean;
  fetchStatus: () => Promise<void>;
  fetchDashboard: () => Promise<void>;
  fetchCost: () => Promise<void>;
}

export const useDaemonStore = create<DaemonState>((set) => ({
  status: null,
  dashboard: null,
  cost: null,
  loading: false,

  fetchStatus: async () => {
    try {
      const data = await api.getStatus();
      set({ status: data });
    } catch {
      set({ status: null });
    }
  },

  fetchDashboard: async () => {
    set({ loading: true });
    try {
      const data = await api.getDashboard();
      set({ dashboard: data, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  fetchCost: async () => {
    try {
      const data = await api.getCost();
      set({ cost: data });
    } catch {
      // ignore
    }
  },
}));
