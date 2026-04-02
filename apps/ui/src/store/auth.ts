import { create } from "zustand";
import { api } from "@/lib/api";

interface AuthState {
  token: string | null;
  loading: boolean;
  error: string | null;
  login: (secret: string) => Promise<boolean>;
  logout: () => void;
  isAuthenticated: () => boolean;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  token: localStorage.getItem("aeqi_token"),
  loading: false,
  error: null,

  login: async (secret: string) => {
    set({ loading: true, error: null });
    try {
      const resp = await api.login(secret);
      if (resp.ok && resp.token) {
        localStorage.setItem("aeqi_token", resp.token);
        set({ token: resp.token, loading: false });
        return true;
      }
      set({ loading: false, error: "Invalid secret" });
      return false;
    } catch {
      set({ loading: false, error: "Login failed" });
      return false;
    }
  },

  logout: () => {
    localStorage.removeItem("aeqi_token");
    set({ token: null });
  },

  isAuthenticated: () => !!get().token,
}));
