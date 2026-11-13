import { useEffect, useState } from "react";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { CopyIcon, FilmIcon, UploadIcon } from "../Icons";
import type { VoiceState } from "../types";

const AUDIO_EXTENSIONS = ["wav", "mp3", "flac", "ogg", "m4a", "aac", "opus", "webm"];

export function FilesView({
  state,
  progress,
  text,
  onStart,
  onCancel,
  onCopy,
}: {
  state: VoiceState;
  progress: number | null;
  text: string;
  onStart: (path: string) => void;
  onCancel: () => void;
  onCopy: (text: string) => void;
}) {
  const [dragging, setDragging] = useState(false);
  const [manualPath, setManualPath] = useState("");

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === "enter" || event.payload.type === "over") {
          setDragging(true);
        } else if (event.payload.type === "leave") {
          setDragging(false);
        } else if (event.payload.type === "drop") {
          setDragging(false);
          const path = event.payload.paths.find((candidate) =>
            AUDIO_EXTENSIONS.some((extension) => candidate.toLowerCase().endsWith(`.${extension}`)),
          );
          if (path && state === "idle") onStart(path);
        }
      })
      .then((stop) => {
        unlisten = stop;
      });
    return () => unlisten?.();
  }, [state, onStart]);

  const pick = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "音频", extensions: AUDIO_EXTENSIONS }],
    });
    if (typeof selected === "string") onStart(selected);
  };

  const busy = state !== "idle";

  return (
    <div className="view files-view">
      <header className="view-header">
        <div>
          <h1>文件转写</h1>
          <p className="view-subtitle">拖入音频，按实时节奏送入识别</p>
        </div>
      </header>

      {!busy && !text ? (
        <button
          className={`dropzone ${dragging ? "is-dragging" : ""}`}
          onClick={pick}
          type="button"
        >
          <span className="dropzone-icon">
            <UploadIcon size={26} />
          </span>
          <strong>拖入音频文件</strong>
          <span>或点击选择 · wav / mp3 / flac / m4a</span>
        </button>
      ) : null}

      <div className="path-row">
        <input
          className="text-input"
          value={manualPath}
          onChange={(event) => setManualPath(event.target.value)}
          placeholder="或粘贴文件路径"
          spellCheck={false}
          onKeyDown={(event) => {
            if (event.key === "Enter" && manualPath.trim() && !busy) {
              onStart(manualPath.trim());
              setManualPath("");
            }
          }}
        />
        <button
          className="ghost-button"
          disabled={!manualPath.trim() || busy}
          onClick={() => {
            onStart(manualPath.trim());
            setManualPath("");
          }}
        >
          转写
        </button>
      </div>

      {busy ? (
        <section className="file-progress-card">
          <div className="file-progress-head">
            <FilmIcon size={18} />
            <span>{state === "requesting" ? "准备中…" : state === "finishing" ? "收尾中…" : "正在转写"}</span>
            <button className="text-button danger" onClick={onCancel}>
              取消
            </button>
          </div>
          <div className="progress-track">
            <span style={{ width: `${Math.max(2, progress ?? 0)}%` }} />
          </div>
          <p className="progress-label">{Math.round(progress ?? 0)}%</p>
        </section>
      ) : null}

      {text ? (
        <section className="result-card">
          <header className="result-head">
            <span>转写结果</span>
            <button className="ghost-button" onClick={() => onCopy(text)}>
              <CopyIcon size={15} />
              复制
            </button>
          </header>
          <p className="result-text">{text}</p>
          {!busy ? (
            <footer className="file-actions">
              <button className="ghost-button" onClick={pick}>
                再转一个
              </button>
            </footer>
          ) : null}
        </section>
      ) : null}
    </div>
  );
}
