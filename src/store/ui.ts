import { create } from "zustand";

export type LayoutMode = "focus" | "split" | "stack";

interface UIState {
  sidebarCollapsed: boolean;
  layout: LayoutMode;
  layoutPickerOpen: boolean;
  splitRatio: number; // 0.0 - 1.0, how much space the main panel gets
  toggleSidebar: () => void;
  setLayout: (mode: LayoutMode) => void;
  setSplitRatio: (ratio: number) => void;
  toggleLayoutPicker: () => void;
  closeLayoutPicker: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  sidebarCollapsed: localStorage.getItem("sigil_sidebar_collapsed") === "true",
  layout: (localStorage.getItem("sigil_layout") as LayoutMode) || "split",
  layoutPickerOpen: false,
  splitRatio: parseFloat(localStorage.getItem("sigil_split_ratio") || "0.65"),
  toggleSidebar: () =>
    set((state) => {
      const next = !state.sidebarCollapsed;
      localStorage.setItem("sigil_sidebar_collapsed", String(next));
      return { sidebarCollapsed: next };
    }),
  setLayout: (mode) => {
    localStorage.setItem("sigil_layout", mode);
    set({ layout: mode, layoutPickerOpen: false });
  },
  setSplitRatio: (ratio) => {
    const clamped = Math.max(0.3, Math.min(0.85, ratio));
    localStorage.setItem("sigil_split_ratio", String(clamped));
    set({ splitRatio: clamped });
  },
  toggleLayoutPicker: () => set((s) => ({ layoutPickerOpen: !s.layoutPickerOpen })),
  closeLayoutPicker: () => set({ layoutPickerOpen: false }),
}));
