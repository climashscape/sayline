import { useEffect, useMemo, useState } from "react";
import { CheckIcon, CloseIcon, RefreshIcon } from "../Icons";
import type { Settings, ZCodeStatus } from "../types";

export function SettingsView({
  settings,
  zcode,
  onSave,
  onRefreshZcode,
}: {
  settings: Settings;
  zcode: ZCodeStatus | null;
  onSave: (value: Settings) => Promise<void>;
  onRefreshZcode: () => void;
}) {
  const [draft, setDraft] = useState<Settings>(settings);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  const dirty = useMemo(
    () => JSON.stringify(draft) !== JSON.stringify(settings),
    [draft, settings],
  );

  const patch = (value: Partial<Settings>) => setDraft((current) => ({ ...current, ...value }));

  const save = async () => {
    setSaving(true);
    try {
      await onSave(draft);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="view settings-view">
      <header className="view-header">
        <div>
          <h1>设置</h1>
          <p className="view-subtitle">热键、语言与后处理规则</p>
        </div>
        <button className="primary-button" onClick={save} disabled={!dirty || saving}>
          {saving ? <span className="spinner spinner-dark" /> : <CheckIcon size={16} />}
          保存
        </button>
      </header>

      <section className="settings-card">
        <h2>外观</h2>
        <div className="field-row">
          <div className="field-label">
            <span>主题</span>
            <small>默认跟随系统，也可固定浅色或深色</small>
          </div>
          <div className="segmented">
            <button
              className={draft.theme === "system" ? "is-selected" : ""}
              onClick={() => patch({ theme: "system" })}
            >
              跟随系统
            </button>
            <button
              className={draft.theme === "light" ? "is-selected" : ""}
              onClick={() => patch({ theme: "light" })}
            >
              浅色
            </button>
            <button
              className={draft.theme === "dark" ? "is-selected" : ""}
              onClick={() => patch({ theme: "dark" })}
            >
              深色
            </button>
          </div>
        </div>
      </section>

      <section className="settings-card">
        <h2>录音</h2>
        <div className="field-row">
          <div className="field-label">
            <span>全局热键</span>
            <small>按住说话，松开识别</small>
          </div>
          <input
            className="text-input hotkey-input"
            value={draft.hotkey}
            onChange={(event) => patch({ hotkey: event.target.value })}
            placeholder="Ctrl+Alt+Space"
            spellCheck={false}
          />
        </div>
        <div className="field-row">
          <div className="field-label">
            <span>识别语言</span>
            <small>跟随服务端支持的两种语言</small>
          </div>
          <div className="segmented">
            <button
              className={draft.language === "zh-CN" ? "is-selected" : ""}
              onClick={() => patch({ language: "zh-CN" })}
            >
              中文
            </button>
            <button
              className={draft.language === "en-US" ? "is-selected" : ""}
              onClick={() => patch({ language: "en-US" })}
            >
              English
            </button>
          </div>
        </div>
        <label className="field-row toggle-row">
          <div className="field-label">
            <span>识别后自动粘贴</span>
            <small>把最终文字发送到当前焦点输入框</small>
          </div>
          <span className={`switch ${draft.auto_paste ? "is-on" : ""}`}>
            <input
              type="checkbox"
              checked={draft.auto_paste}
              onChange={(event) => patch({ auto_paste: event.target.checked })}
            />
            <span />
          </span>
        </label>
      </section>

      <section className="settings-card">
        <h2>转写</h2>
        <div className="field-row">
          <div className="field-label">
            <span>文件转写速度</span>
            <small>越快越省时，过高可能被服务端限流</small>
          </div>
          <select
            className="text-input select-input"
            value={draft.file_speed}
            onChange={(event) => patch({ file_speed: Number(event.target.value) })}
          >
            <option value={1}>1× 实时</option>
            <option value={2}>2×</option>
            <option value={3}>3×</option>
            <option value={4}>4×</option>
          </select>
        </div>
        <div className="field-block">
          <div className="field-label">
            <span>替换规则</span>
            <small>按顺序替换最终文本，长词优先</small>
          </div>
          <div className="replacement-list">
            {draft.replacements.length === 0 ? (
              <p className="muted">还没有规则</p>
            ) : (
              draft.replacements.map((rule, index) => (
                <div className="replacement-row" key={index}>
                  <input
                    className="text-input"
                    value={rule.from}
                    placeholder="原文"
                    onChange={(event) => {
                      const next = [...draft.replacements];
                      next[index] = { ...next[index], from: event.target.value };
                      patch({ replacements: next });
                    }}
                  />
                  <span className="arrow">→</span>
                  <input
                    className="text-input"
                    value={rule.to}
                    placeholder="替换为"
                    onChange={(event) => {
                      const next = [...draft.replacements];
                      next[index] = { ...next[index], to: event.target.value };
                      patch({ replacements: next });
                    }}
                  />
                  <button
                    className="icon-button danger"
                    aria-label="删除规则"
                    onClick={() =>
                      patch({
                        replacements: draft.replacements.filter((_, item) => item !== index),
                      })
                    }
                  >
                    <CloseIcon size={14} />
                  </button>
                </div>
              ))
            )}
            <button
              className="ghost-button"
              onClick={() =>
                patch({
                  replacements: [...draft.replacements, { from: "", to: "" }],
                })
              }
            >
              添加规则
            </button>
          </div>
        </div>
      </section>

      <section className="settings-card">
        <div className="card-title-row">
          <h2>ZCode 连接</h2>
          <button className="icon-button" onClick={onRefreshZcode} aria-label="重新检查">
            <RefreshIcon size={15} />
          </button>
        </div>
        <div className={`zcode-card ${zcode?.connected ? "is-connected" : ""}`}>
          <span className="status-dot" />
          <div>
            <strong>{zcode?.connected ? "已连接" : "未连接"}</strong>
            <p>{zcode?.message ?? "正在检查…"}</p>
            {zcode?.expires_at ? <small>登录有效期至 {zcode.expires_at}</small> : null}
          </div>
        </div>
        <div className="field-row">
          <div className="field-label">
            <span>服务地址</span>
            <small>默认 https://zcode.z.ai</small>
          </div>
          <input
            className="text-input endpoint-input"
            value={draft.endpoint_origin}
            onChange={(event) => patch({ endpoint_origin: event.target.value })}
            spellCheck={false}
          />
        </div>
      </section>
    </div>
  );
}
