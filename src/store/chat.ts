import { create } from "zustand";
import type { ChatThreadState } from "@/lib/types";

const CHANNEL_STORAGE_KEY = "sigil_chat_channel";
const THREADS_STORAGE_KEY = "sigil_chat_threads_v2";
const GLOBAL_CHANNEL_KEY = "__global__";

function channelKey(channel: string | null): string {
  return channel || GLOBAL_CHANNEL_KEY;
}

function readChannel(): string | null {
  try {
    return localStorage.getItem(CHANNEL_STORAGE_KEY);
  } catch {
    return null;
  }
}

function readThreads(): Record<string, ChatThreadState> {
  try {
    const raw = localStorage.getItem(THREADS_STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw) as Record<string, ChatThreadState>;
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

function persistChannel(channel: string | null) {
  try {
    if (channel) localStorage.setItem(CHANNEL_STORAGE_KEY, channel);
    else localStorage.removeItem(CHANNEL_STORAGE_KEY);
  } catch {
    // ignore localStorage failures
  }
}

function persistThreads(threads: Record<string, ChatThreadState>) {
  try {
    localStorage.setItem(THREADS_STORAGE_KEY, JSON.stringify(threads));
  } catch {
    // ignore localStorage failures
  }
}

interface ChatState {
  channel: string | null;
  threads: Record<string, ChatThreadState>;
  setChannel: (ch: string | null) => void;
  getOrCreateThread: (channel: string | null) => ChatThreadState;
  updateThread: (channel: string | null, patch: Partial<ChatThreadState>) => void;
}

export const useChatStore = create<ChatState>((set, get) => ({
  channel: readChannel(),
  threads: readThreads(),
  setChannel: (ch) => {
    persistChannel(ch);
    set({ channel: ch });
  },
  getOrCreateThread: (channel): ChatThreadState => {
    const key = channelKey(channel);
    const current = get().threads[key];
    if (current) {
      return current;
    }

    const next = {};
    set((state) => {
      const threads = { ...state.threads, [key]: next };
      persistThreads(threads);
      return { threads };
    });
    return next;
  },
  updateThread: (channel, patch) => {
    const key = channelKey(channel);
    set((state) => {
      const current = state.threads[key] || {};
      const threads = {
        ...state.threads,
        [key]: { ...current, ...patch },
      };
      persistThreads(threads);
      return { threads };
    });
  },
}));
