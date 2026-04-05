import { create } from "zustand";
import { api } from "@/lib/api";

export type AuthMode = "none" | "secret" | "accounts" | null;

interface User {
  id: string;
  email: string;
  name: string;
  avatar_url?: string;
}

interface AuthState {
  token: string | null;
  authMode: AuthMode;
  googleOAuth: boolean;
  user: User | null;
  loading: boolean;
  error: string | null;

  fetchAuthMode: () => Promise<void>;
  login: (secret: string) => Promise<boolean>;
  loginWithEmail: (email: string, password: string) => Promise<boolean>;
  signup: (email: string, password: string, name: string) => Promise<boolean>;
  handleOAuthCallback: (token: string) => void;
  fetchMe: () => Promise<void>;
  logout: () => void;
  isAuthenticated: () => boolean;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  token: localStorage.getItem("aeqi_token"),
  authMode: (localStorage.getItem("aeqi_auth_mode") as AuthMode) || null,
  googleOAuth: false,
  user: null,
  loading: false,
  error: null,

  fetchAuthMode: async () => {
    try {
      const resp = await api.getAuthMode();
      const mode = (resp.mode || "secret") as AuthMode;
      localStorage.setItem("aeqi_auth_mode", mode || "secret");
      set({ authMode: mode, googleOAuth: resp.google_oauth });

      // In none mode, auto-generate a token if needed.
      if (mode === "none" && !get().token) {
        try {
          const loginResp = await api.login("");
          if (loginResp.ok && loginResp.token) {
            localStorage.setItem("aeqi_token", loginResp.token);
            set({ token: loginResp.token });
          }
        } catch {
          // In none mode, this is fine — routes don't require auth.
          set({ token: "none" });
        }
      }
    } catch {
      // If endpoint not available, assume secret mode (backward compat).
      set({ authMode: "secret" });
    }
  },

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

  loginWithEmail: async (email: string, password: string) => {
    set({ loading: true, error: null });
    try {
      const resp = await api.loginWithEmail(email, password);
      if (resp.ok && resp.token) {
        localStorage.setItem("aeqi_token", resp.token);
        set({
          token: resp.token,
          user: resp.user || null,
          loading: false,
        });
        return true;
      }
      set({ loading: false, error: "Invalid email or password" });
      return false;
    } catch (e: any) {
      set({ loading: false, error: e?.message || "Login failed" });
      return false;
    }
  },

  signup: async (email: string, password: string, name: string) => {
    set({ loading: true, error: null });
    try {
      const resp = await api.signup(email, password, name);
      if (resp.ok && resp.token) {
        localStorage.setItem("aeqi_token", resp.token);
        set({
          token: resp.token,
          user: resp.user || null,
          loading: false,
        });
        return true;
      }
      set({ loading: false, error: "Signup failed" });
      return false;
    } catch (e: any) {
      set({ loading: false, error: e?.message || "Signup failed" });
      return false;
    }
  },

  handleOAuthCallback: (token: string) => {
    localStorage.setItem("aeqi_token", token);
    set({ token });
  },

  fetchMe: async () => {
    try {
      const user = await api.getMe();
      set({ user });
    } catch {
      // Not critical.
    }
  },

  logout: () => {
    localStorage.removeItem("aeqi_token");
    set({ token: null, user: null });
  },

  isAuthenticated: () => {
    const { authMode, token } = get();
    if (authMode === "none") return true;
    return !!token;
  },
}));
