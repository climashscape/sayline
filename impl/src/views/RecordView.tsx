import { AlertIcon, CloseIcon, CopyIcon, MicIcon, RefreshIcon, StopIcon } from "../Icons";
import { Waveform } from "../components/Waveform";
import type { VoiceState, ZCodeStatus } from "../types";

const STATE_LABEL: Record<VoiceState, string> = {
  idle: "待命",
  requesting: "连接中",
  recording: "正在听",
  finishing: "识别中",
};

const ZERO_LEVELS = Array<number>(10).fill(0);

export function RecordView({
  state,
  levels,
  partial,
  finalText,
  error,
  hotkey,
  zcode,
  onToggle,
  onCancel,
  onClearError,
  onRefreshZcode,
  onCopy,
  onClear,
}: {
  state: VoiceState;
  levels: number[];
  partial: string;
  finalText: string;
  error: { message: string; code: string } | null;
  hotkey: string;
  zcode: ZCodeStatus | null;
  onToggle: () => void;
  onCancel: () => void;
  onClearError: () => void;
  onRefreshZcode: () => void;
  onCopy: (text: string) => void;
  onClear: () => void;
}) {
  const text = partial || finalText;
  const active = state !== "idle";
  const recording = state === "recording";
  const waiting = state === "requesting" || state === "finishing";

  const hint =
    state === "idle"
      ? "按住热键或点击麦克风开始"
      : state === "requesting"
        ? "正在连接语音识别…"
        : state === "recording"
          ? "再次点击或松开热键结束"
          : "正在整理文字…";

  return (
    <div className="view record-view">
      <header className="view-header">
        <div>
          <h1>语音输入</h1>
          <p className="view-subtitle">按住说话，松开后自动写入当前输入框</p>
        </div>
        <span className={`status-pill status-${state}`}>
          <span className="status-dot" />
          {STATE_LABEL[state]}
        </span>
      </header>

      {zcode && !zcode.connected ? (
        <div className="banner banner-warn">
          <AlertIcon size={15} />
          <span>{zcode.message}</span>
          <button className="text-button" onClick={onRefreshZcode}>
            <RefreshIcon size={13} />
            重新检查
          </button>
        </div>
      ) : null}

      {error ? (
        <div className="banner banner-error">
          <AlertIcon size={15} />
          <span>{error.message}</span>
          <button className="icon-button" onClick={onClearError} aria-label="关闭错误">
            <CloseIcon size={13} />
          </button>
        </div>
      ) : null}

      <section className={`voice-console ${active ? "is-active" : ""}`}>
        <div className="voice-preview">
          <div className="voice-preview-head">
            <span>{partial ? "实时转写" : text ? "转写结果" : "语音输入"}</span>
            {text ? (
              <span className="preview-actions">
                <button className="text-button" onClick={() => onCopy(text)}>
                  <CopyIcon size={13} />
                  复制
                </button>
                {!partial ? (
                  <button className="text-button" onClick={onClear}>
                    清空
                  </button>
                ) : null}
              </span>
            ) : null}
          </div>
          {text ? (
            <p className={`voice-preview-text ${partial ? "is-partial" : ""}`}>{text}</p>
          ) : (
            <p className="voice-preview-placeholder">
              按住 <kbd>{hotkey}</kbd> 说话，识别结果会出现在这里。
            </p>
          )}
        </div>

        <footer className="voice-console-foot">
          <div className="voice-meta">
            <span
              className={`status-dot ${
                recording ? "is-recording" : waiting ? "is-connecting" : ""
              }`}
            />
            <span>{STATE_LABEL[state]}</span>
            <span className="voice-hint">{hint}</span>
          </div>

          <div className="voice-control">
            {active ? (
              <div className="voice-pill">
                <Waveform levels={recording ? levels : ZERO_LEVELS} bars={10} />
                {recording ? (
                  <button
                    type="button"
                    className="voice-stop"
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={onToggle}
                    aria-label="停止录音"
                    aria-pressed="true"
                    title="停止录音"
                  >
                    <span className="voice-stop-square" />
                  </button>
                ) : (
                  <button
                    type="button"
                    className="voice-stop"
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={state === "requesting" ? onCancel : undefined}
                    aria-label={state === "requesting" ? "取消连接" : "正在识别"}
                    aria-disabled={state === "finishing" || undefined}
                    title={state === "requesting" ? "取消连接" : "正在识别"}
                  >
                    <span className="voice-spinner" />
                  </button>
                )}
              </div>
            ) : (
              <button
                type="button"
                className="voice-idle"
                onMouseDown={(event) => event.preventDefault()}
                onClick={onToggle}
                aria-label="开始录音"
                aria-pressed="false"
                title={`开始录音 · ${hotkey}`}
              >
                <MicIcon size={16} />
              </button>
            )}
          </div>
        </footer>
      </section>
    </div>
  );
}
