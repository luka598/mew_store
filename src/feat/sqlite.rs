use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;
use crate::store;

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn new() -> rusqlite::Result<Self> {
        let conn = Connection::open("store.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn get(&self, key: &str) -> Result<String, ()> {
        self.conn.lock().unwrap()
            .query_row("SELECT value FROM kv WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(|_| ())?
            .ok_or(())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), ()> {
        self.conn.lock().unwrap()
            .execute(
                "INSERT INTO kv (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                (key, value),
            )
            .map(|_| ())
            .map_err(|_| ())
    }

    fn del(&self, key: &str) -> Result<(), ()> {
        self.conn.lock().unwrap()
            .execute("DELETE FROM kv WHERE key = ?1", [key])
            .map(|_| ())
            .map_err(|_| ())
    }

    fn list(&self, pattern: &str, cursor: i64, limit: i64) -> Result<Vec<String>, ()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT key FROM kv WHERE key GLOB ?1 ORDER BY key LIMIT ?2 OFFSET ?3")
            .map_err(|_| ())?;
        let rows = stmt
            .query_map(rusqlite::params![pattern, limit, cursor], |r| {
                r.get::<_, String>(0)
            })
            .map_err(|_| ())?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|_| ())?);
        }
        Ok(out)
    }
}

static STORE_HANDLER: std::sync::OnceLock<SqliteStore> = std::sync::OnceLock::new();

pub fn handler(op: store::Operation) -> Result<Vec<String>, String> {
    let s = STORE_HANDLER.get_or_init(|| SqliteStore::new().expect("sqlite open failed"));
    match op {
        store::Operation::Get { key } => s
            .get(&key)
            .map(|v| vec![v])
            .map_err(|_| "not_found".to_string()),
        store::Operation::Set { key, value } => s
            .set(&key, &value)
            .map(|_| Vec::new())
            .map_err(|_| "store_error".to_string()),
        store::Operation::Del { key } => s
            .del(&key)
            .map(|_| Vec::new())
            .map_err(|_| "store_error".to_string()),
        store::Operation::List { pattern, cursor, limit } => s
            .list(&pattern, cursor as i64, limit as i64)
            .map_err(|_| "store_error".to_string()),
    }
}

pub struct SqliteStoreMiddleware;

impl store::Middleware for SqliteStoreMiddleware {
    fn before(&self, req: store::Request) -> store::Flow {
        match req {
            store::Request::Exec { op, .. } => match handler(op) {
                Ok(values) => store::Flow::Return(store::Response::Values(values)),
                Err(e) => store::Flow::Return(store::Response::Error(e)),
            },
            store::Request::None => store::Flow::Return(store::Response::Values(Vec::new())),
        }
    }
}