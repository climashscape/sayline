import { useCallback, useEffect, useRef, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import { api } from "./api";
import { ClockIcon, FilmIcon, MicIcon, SettingsIcon } from "./Icons";
import { Toast, type ToastData, type ToastTone } from "./components/Toast";
import { FilesView } from "./views/FilesView";
import { HistoryView } from "./views/HistoryView";
import { RecordView } from "./views/RecordView";
import { SettingsView } from "./views/SettingsView";
import type { SaylineEvent, Settings, VoiceState, ZCodeStatus } from "./types";

type View = "record" | "history" | "files" | "settings";

const LEVEL_SLOTS = 10;

function resolveTheme(theme: string): "light" | "dark" {
  if (theme === "light" || theme === "dark") return theme;
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function applyTheme(theme: string) {
  const resolved = resolveTheme(theme);
  document.documentElement.dataset.theme = resolved;
  window.localStorage.setItem("sayline-theme", resolved);
  void emit("sayline-theme", resolved).catch(() => undefined);
}

const NAV: Array<{ id: View; label: string; icon: typeof MicIcon }> = [
  { id: "record", label: "录音", icon: MicIcon },
  { id: "history", label: "历史", icon: ClockIcon },
  { id: "files", label: "文件", icon: FilmIcon },
  { id: "settings", label: "设置", icon: SettingsIcon },
];

export default function App() {
  const [view, setView] = useState<View>("record");
  const [micState, setMicState] = useState<VoiceState>("idle");
  const [fileState, setFileState] = useState<VoiceState>("idle");
  const [levels, setLevels] = useState<number[]>(() => Array(LEVEL_SLOTS).fill(0));
  const [partial, setPartial] = useState("");
  const [finalText, setFinalText] = useState("");
  const [error, setError] = useState<{ message: string; code: string } | null>(null);
  const [fileProgress, setFileProgress] = useState<number | null>(null);
  const [fileText, setFileText] = useState("");
  const [historyVersion, setHistoryVersion] = useState(0);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [zcode, setZcode] = useState<ZCodeStatus | null>(null);
  const [toast, setToast] = useState<ToastData | null>(null);
  const toastId = useRef(0);

  const pushToast = useCallback((message: string, tone: ToastTone = "info") => {
    toastId.current += 1;
    setToast({ id: toastId.current, message, tone });
  }, []);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 4200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  const refreshZcode = useCallback(() => {
    api
      .zcodeStatus()
      .then(setZcode)
      .catch((reason: unknown) =>
        setZcode({
          connected: false,
          expires_at: null,
          message: String(reason),
        }),
      );
  }, []);

  useEffect(() => {
    api.getSettings().then(setSettings).catch((reason) => pushToast(String(reason), "error"));
    refreshZcode();
  }, [pushToast, refreshZcode]);

  useEffect(() => {
    if (settings) applyTheme(settings.theme);
  }, [settings?.theme]);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => {
      if (!settings || settings.theme === "system") applyTheme("system");
    };
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, [settings?.theme]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<SaylineEvent>("sayline", (event) => {
      const payload = event.payload;
      switch (payload.event) {
        case "state":
          if (payload.source === "mic") {
            if (payload.state === "requesting") {
              setFinalText("");
              setPartial("");
              setError(null);
            }
            setMicState(payload.state);
            if (payload.state === "idle") {
              setLevels(Array(LEVEL_SLOTS).fill(0));
            }
          } else {
            setFileState(payload.state);
            if (payload.state === "idle") setFileProgress(null);
          }
          break;
        case "level":
          if (payload.source === "mic") {
            setLevels((current) => [...current.slice(-(LEVEL_SLOTS - 1)), payload.level]);
          }
          break;
        case "partial":
          if (payload.source === "mic") setPartial(payload.text);
          else setFileText(payload.text);
          break;
        case "final":
          setHistoryVersion((value) => value + 1);
          if (payload.source === "mic") {
            setFinalText(payload.text);
            setPartial("");
            setMicState("idle");
            setLevels(Array(LEVEL_SLOTS).fill(0));
            pushToast("识别完成", "success");
          } else {
            setFileText(payload.text);
            setFileState("idle");
            setFileProgress(null);
            pushToast("文件转写完成", "success");
          }
          break;
        case "error":
          if (payload.source === "mic") {
            setMicState("idle");
            setError({ message: payload.message, code: payload.code });
          } else {
            setFileState("idle");
            setFileProgress(null);
          }
          pushToast(payload.message, "error");
          break;
        case "progress":
          if (payload.source === "file") setFileProgress(payload.percent);
          break;
      }
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [pushToast]);

  const toggleMic = useCallback(() => {
    if (micState === "idle") {
      setFinalText("");
      setPartial("");
      setMicState("requesting");
      setError(null);
      api.startRecording().catch((reason: unknown) => {
        setMicState("idle");
        pushToast(String(reason), "error");
      });
      return;
    }
    if (micState === "recording") {
      setMicState("finishing");
      api.stopRecording().catch((reason: unknown) => pushToast(String(reason), "error"));
      return;
    }
    if (micState === "requesting") {
      api.cancelRecording().catch(() => undefined);
      setMicState("idle");
    }
  }, [micState, pushToast]);

  const cancelMic = useCallback(() => {
    api.cancelRecording().catch(() => undefined);
    setMicState("idle");
  }, []);

  const copy = useCallback(
    (text: string) => {
      api
        .copyText(text)
        .then(() => pushToast("已复制到剪贴板", "success"))
        .catch((reason: unknown) => pushToast(String(reason), "error"));
    },
    [pushToast],
  );

  const saveSettings = useCallback(
    async (value: Settings) => {
      try {
        await api.saveSettings(value);
        setSettings(value);
        pushToast("设置已保存", "success");
      } catch (reason) {
        pushToast(String(reason), "error");
        throw reason;
      }
    },
    [pushToast],
  );

  const startFile = useCallback(
    (path: string) => {
      setFileText("");
      setFileProgress(0);
      setFileState("requesting");
      api.transcribeFile(path).catch((reason: unknown) => {
        setFileState("idle");
        setFileProgress(null);
        pushToast(String(reason), "error");
      });
    },
    [pushToast],
  );

  const cancelFile = useCallback(() => {
    api.cancelFile().catch(() => undefined);
    setFileState("idle");
    setFileProgress(null);
  }, []);

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">S</span>
          <div className="brand-text">
            <strong>Sayline</strong>
            <span>voice input</span>
          </div>
        </div>

        <nav className="nav">
          {NAV.map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                className={`nav-item ${view === item.id ? "is-active" : ""}`}
                onClick={() => setView(item.id)}
              >
                <Icon size={18} />
                <span>{item.label}</span>
                {item.id === "record" && micState !== "idle" ? (
                  <span className="nav-live" />
                ) : null}
              </button>
            );
          })}
        </nav>

        <div className="sidebar-foot">
          <button className={`connection-chip ${zcode?.connected ? "is-on" : ""}`} onClick={refreshZcode}>
            <span className="status-dot" />
            {zcode?.connected ? "ZCode 已连接" : "ZCode 未连接"}
          </button>
          {settings ? (
            <div className="hotkey-chip">
              <span>按住</span>
              <kbd>{settings.hotkey}</kbd>
            </div>
          ) : null}
        </div>
      </aside>

      <main className="content">
        {view === "record" ? (
          <RecordView
            state={micState}
            levels={levels}
            partial={partial}
            finalText={finalText}
            error={error}
            hotkey={settings?.hotkey ?? "Ctrl+Alt+Space"}
            zcode={zcode}
            onToggle={toggleMic}
            onCancel={cancelMic}
            onClearError={() => setError(null)}
            onRefreshZcode={refreshZcode}
            onCopy={copy}
            onClear={() => {
              setFinalText("");
              setPartial("");
            }}
          />
        ) : null}
        {view === "history" ? (
          <HistoryView
            version={historyVersion}
            onToast={pushToast}
            onGoRecord={() => setView("record")}
          />
        ) : null}
        {view === "files" ? (
          <FilesView
            state={fileState}
            progress={fileProgress}
            text={fileText}
            onStart={startFile}
            onCancel={cancelFile}
            onCopy={copy}
          />
        ) : null}
        {view === "settings" && settings ? (
          <SettingsView
            settings={settings}
            zcode={zcode}
            onSave={saveSettings}
            onRefreshZcode={refreshZcode}
          />
        ) : null}
        {view === "settings" && !settings ? (
          <div className="empty-state">正在读取设置…</div>
        ) : null}
      </main>

      <Toast toast={toast} onClose={() => setToast(null)} />
    </div>
  );
}
