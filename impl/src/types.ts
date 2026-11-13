export type Language = "zh-CN" | "en-US";
export type Theme = "system" | "light" | "dark";
export type VoiceState = "idle" | "requesting" | "recording" | "finishing";

export interface Replacement {
  from: string;
  to: string;
}

export interface Settings {
  hotkey: string;
  language: Language;
  theme: Theme;
  auto_paste: boolean;
  endpoint_origin: string;
  replacements: Replacement[];
  paste_delay_ms: number;
  file_speed: number;
}

export interface HistoryItem {
  id: number;
  created_at: string;
  text: string;
  language: string;
  duration_ms: number;
  source: string;
  title: string | null;
}

export interface ZCodeStatus {
  connected: boolean;
  expires_at: string | null;
  message: string;
}

export type SaylineEvent =
  | { event: "state"; state: VoiceState; source: "mic" | "file" }
  | { event: "level"; level: number; source: string }
  | { event: "partial"; text: string; source: string }
  | { event: "final"; text: string; source: string; item: HistoryItem | null }
  | { event: "error"; message: string; source: string; code: string }
  | { event: "progress"; percent: number; source: string };
