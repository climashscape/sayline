import { invoke } from "@tauri-apps/api/core";
import type { HistoryItem, Settings, ZCodeStatus } from "./types";

export const api = {
  startRecording: () => invoke<void>("start_recording"),
  stopRecording: () => invoke<void>("stop_recording"),
  cancelRecording: () => invoke<void>("cancel_recording"),
  recordingActive: () => invoke<boolean>("recording_active"),
  zcodeStatus: () => invoke<ZCodeStatus>("zcode_status"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (value: Settings) => invoke<void>("save_settings", { value }),
  listHistory: (query?: string) =>
    invoke<HistoryItem[]>("list_history", { query: query?.trim() ? query : null }),
  deleteHistory: (id: number) => invoke<void>("delete_history", { id }),
  clearHistory: () => invoke<void>("clear_history"),
  exportHistory: (format: "md" | "json") =>
    invoke<string>("export_history", { format }),
  copyText: (text: string) => invoke<void>("copy_text", { text }),
  transcribeFile: (path: string) => invoke<void>("transcribe_file", { path }),
  cancelFile: () => invoke<void>("cancel_file"),
  showMainWindow: () => invoke<void>("show_main_window"),
};
