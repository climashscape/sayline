import { useEffect, useState } from "react";
import { api } from "../api";
import { ClockIcon, CopyIcon, DownloadIcon, SearchIcon, TrashIcon } from "../Icons";
import type { HistoryItem } from "../types";

function formatTime(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function HistoryView({
  version,
  onToast,
  onGoRecord,
}: {
  version: number;
  onToast: (message: string, tone?: "info" | "success" | "error") => void;
  onGoRecord: () => void;
}) {
  const [query, setQuery] = useState("");
  const [items, setItems] = useState<HistoryItem[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const timer = window.setTimeout(() => {
      setLoading(true);
      api
        .listHistory(query)
        .then((result) => {
          if (!cancelled) setItems(result);
        })
        .catch((error: unknown) => {
          if (!cancelled) onToast(String(error), "error");
        })
        .finally(() => {
          if (!cancelled) setLoading(false);
        });
    }, 180);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [query, version]);

  const copy = async (text: string) => {
    try {
      await api.copyText(text);
      onToast("已复制到剪贴板", "success");
    } catch (error) {
      onToast(String(error), "error");
    }
  };

  const remove = async (id: number) => {
    if (!window.confirm("删除这条记录？")) return;
    try {
      await api.deleteHistory(id);
      setItems((current) => current.filter((item) => item.id !== id));
      onToast("已删除", "success");
    } catch (error) {
      onToast(String(error), "error");
    }
  };

  const exportAs = async (format: "md" | "json") => {
    try {
      const path = await api.exportHistory(format);
      onToast(`已导出到 ${path}`, "success");
    } catch (error) {
      onToast(String(error), "error");
    }
  };

  const clearAll = async () => {
    if (!window.confirm("清空全部转写历史？此操作不可撤销。")) return;
    try {
      await api.clearHistory();
      setItems([]);
      onToast("历史已清空", "success");
    } catch (error) {
      onToast(String(error), "error");
    }
  };

  return (
    <div className="view history-view">
      <header className="view-header">
        <div>
          <h1>历史</h1>
          <p className="view-subtitle">只保存在本机，默认不存音频</p>
        </div>
        <div className="header-actions">
          <button className="ghost-button" onClick={() => exportAs("md")}>
            <DownloadIcon size={15} />
            Markdown
          </button>
          <button className="ghost-button" onClick={() => exportAs("json")}>
            <DownloadIcon size={15} />
            JSON
          </button>
        </div>
      </header>

      <div className="toolbar">
        <label className="search-box">
          <SearchIcon size={16} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索转写内容"
            spellCheck={false}
          />
        </label>
        {items.length > 0 ? (
          <button className="text-button danger" onClick={clearAll}>
            清空全部
          </button>
        ) : null}
      </div>

      {loading ? (
        <div className="empty-state">正在读取…</div>
      ) : items.length === 0 ? (
        <div className="empty-state">
          <ClockIcon size={28} />
          <p>{query ? "没有匹配的记录" : "还没有记录"}</p>
          {!query ? (
            <button className="ghost-button" onClick={onGoRecord}>
              去录一段
            </button>
          ) : null}
        </div>
      ) : (
        <div className="history-list">
          {items.map((item) => (
            <article className="history-card" key={item.id}>
              <header className="history-meta">
                <span className={`source-badge source-${item.source}`}>
                  {item.source === "file" ? "文件" : "麦克风"}
                </span>
                <span>{formatTime(item.created_at)}</span>
                <span>{Math.max(1, Math.round(item.duration_ms / 1000))}s</span>
                {item.title ? <span className="history-title">{item.title}</span> : null}
              </header>
              <p className="history-text">{item.text}</p>
              <footer className="history-actions">
                <button className="icon-button" onClick={() => copy(item.text)} aria-label="复制">
                  <CopyIcon size={15} />
                </button>
                <button
                  className="icon-button danger"
                  onClick={() => remove(item.id)}
                  aria-label="删除"
                >
                  <TrashIcon size={15} />
                </button>
              </footer>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}
