import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { MicIcon } from "./Icons";
import { Waveform } from "./components/Waveform";
import type { SaylineEvent, VoiceState } from "./types";

const LABEL: Record<VoiceState, string> = {
  idle: "Sayline",
  requesting: "正在连接…",
  recording: "正在听…",
  finishing: "正在整理…",
};

export default function Overlay() {
  const [state, setState] = useState<VoiceState>("idle");
  const [levels, setLevels] = useState<number[]>(() => Array(10).fill(0));
  const [text, setText] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    const stored = window.localStorage.getItem("sayline-theme");
    const resolved =
      stored === "light" || stored === "dark"
        ? stored
        : window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light";
    document.documentElement.dataset.theme = resolved;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<string>("sayline-theme", (event) => {
      if (event.payload === "light" || event.payload === "dark") {
        document.documentElement.dataset.theme = event.payload;
      }
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<SaylineEvent>("sayline", (event) => {
      const payload = event.payload;
      if (payload.source !== "mic") return;
      switch (payload.event) {
        case "state":
          setState(payload.state);
          if (payload.state === "requesting") {
            setText("");
            setError("");
          }
          if (payload.state === "recording") setError("");
          if (payload.state === "idle") {
            setLevels(Array(10).fill(0));
          } else {
            const stored = window.localStorage.getItem("sayline-theme");
            if (stored === "light" || stored === "dark") {
              document.documentElement.dataset.theme = stored;
            }
          }
          break;
        case "level":
          setLevels((current) => [...current.slice(-9), payload.level]);
          break;
        case "partial":
          setText(payload.text);
          break;
        case "final":
          setText(payload.text);
          setError("");
          break;
        case "error":
          setError(payload.message);
          setText("");
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
  }, []);

  return (
    <div className="overlay-shell">
      <div className={`overlay-pill overlay-${state} ${error ? "is-error" : ""}`}>
        <span className="overlay-state">
          {state === "recording" ? (
            <span className="overlay-live-dot" />
          ) : state === "requesting" || state === "finishing" ? (
            <span className="spinner" />
          ) : (
            <MicIcon size={15} />
          )}
        </span>
        <Waveform levels={levels} bars={10} className="overlay-wave" />
        <span className="overlay-caption">{error || (text ? text : LABEL[state])}</span>
      </div>
    </div>
  );
}
