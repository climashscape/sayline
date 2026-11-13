use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryItem {
    pub id: i64,
    pub created_at: String,
    pub text: String,
    pub language: String,
    pub duration_ms: i64,
    pub source: String,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewTranscript {
    pub text: String,
    pub language: String,
    pub duration_ms: i64,
    pub source: String,
    pub title: Option<String>,
}

pub struct History {
    conn: Mutex<Connection>,
}

fn database_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sayline")
        .join("history.db")
}

impl History {
    pub fn open() -> Result<Self, String> {
        let path = database_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("无法创建数据目录: {error}"))?;
        }
        let conn = Connection::open(&path).map_err(|error| format!("无法打开历史库: {error}"))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             CREATE TABLE IF NOT EXISTS transcripts (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               created_at TEXT NOT NULL,
               text TEXT NOT NULL,
               language TEXT NOT NULL,
               duration_ms INTEGER NOT NULL DEFAULT 0,
               source TEXT NOT NULL DEFAULT 'mic',
               title TEXT
             );
             CREATE INDEX IF NOT EXISTS idx_transcripts_created
               ON transcripts(created_at DESC);",
        )
        .map_err(|error| format!("无法初始化历史库: {error}"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn insert(&self, item: &NewTranscript) -> Result<HistoryItem, String> {
        let created_at = chrono::Local::now().to_rfc3339();
        let conn = self.conn.lock().map_err(|_| "历史库锁失败".to_string())?;
        conn.execute(
            "INSERT INTO transcripts (created_at, text, language, duration_ms, source, title)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                created_at,
                item.text,
                item.language,
                item.duration_ms,
                item.source,
                item.title
            ],
        )
        .map_err(|error| format!("无法写入历史: {error}"))?;
        let id = conn.last_insert_rowid();
        Ok(HistoryItem {
            id,
            created_at,
            text: item.text.clone(),
            language: item.language.clone(),
            duration_ms: item.duration_ms,
            source: item.source.clone(),
            title: item.title.clone(),
        })
    }

    pub fn list(&self, query: Option<&str>, limit: i64) -> Result<Vec<HistoryItem>, String> {
        let conn = self.conn.lock().map_err(|_| "历史库锁失败".to_string())?;
        let limit = limit.clamp(1, 2000);
        let mut items = Vec::new();
        let like = query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| format!("%{value}%"));
        match like {
            Some(like) => {
                let mut statement = conn
                    .prepare(
                        "SELECT id, created_at, text, language, duration_ms, source, title
                         FROM transcripts
                         WHERE text LIKE ?1 OR COALESCE(title, '') LIKE ?1
                         ORDER BY id DESC LIMIT ?2",
                    )
                    .map_err(|error| error.to_string())?;
                let rows = statement
                    .query_map(params![like, limit], map_row)
                    .map_err(|error| error.to_string())?;
                for row in rows {
                    items.push(row.map_err(|error| error.to_string())?);
                }
            }
            None => {
                let mut statement = conn
                    .prepare(
                        "SELECT id, created_at, text, language, duration_ms, source, title
                         FROM transcripts ORDER BY id DESC LIMIT ?1",
                    )
                    .map_err(|error| error.to_string())?;
                let rows = statement
                    .query_map(params![limit], map_row)
                    .map_err(|error| error.to_string())?;
                for row in rows {
                    items.push(row.map_err(|error| error.to_string())?);
                }
            }
        }
        Ok(items)
    }

    pub fn delete(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|_| "历史库锁失败".to_string())?;
        conn.execute("DELETE FROM transcripts WHERE id = ?1", params![id])
            .map_err(|error| format!("删除失败: {error}"))?;
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|_| "历史库锁失败".to_string())?;
        conn.execute("DELETE FROM transcripts", [])
            .map_err(|error| format!("清空失败: {error}"))?;
        Ok(())
    }

    pub fn export(&self, format: &str) -> Result<String, String> {
        let items = self.list(None, 2000)?;
        match format {
            "json" => serde_json::to_string_pretty(&items).map_err(|error| error.to_string()),
            "md" | "markdown" => {
                let mut body = String::from("# Sayline 转写历史\n\n");
                for item in items {
                    body.push_str(&format!("## {}\n\n", item.created_at));
                    if let Some(title) = &item.title {
                        body.push_str(&format!("- 来源：{}\n", title));
                    }
                    body.push_str(&format!(
                        "- 语言：{} · 时长：{:.1}s\n\n{}\n\n---\n\n",
                        item.language,
                        item.duration_ms as f64 / 1000.0,
                        item.text
                    ));
                }
                Ok(body)
            }
            other => Err(format!("不支持的导出格式: {other}")),
        }
    }
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryItem> {
    Ok(HistoryItem {
        id: row.get(0)?,
        created_at: row.get(1)?,
        text: row.get(2)?,
        language: row.get(3)?,
        duration_ms: row.get(4)?,
        source: row.get(5)?,
        title: row.get(6)?,
    })
}
