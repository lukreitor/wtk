//! SQLite database for tracking token savings.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::PathBuf;

/// Database for tracking command executions and token savings.
pub struct TrackingDb {
    conn: Connection,
}

impl TrackingDb {
    /// Open or create the tracking database.
    pub fn open() -> Result<Self> {
        let db_path = get_db_path()?;

        // Create directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path))?;

        let db = Self { conn };
        db.init_schema()?;
        db.migrate()?;

        Ok(db)
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS commands (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                command TEXT NOT NULL,
                input_chars INTEGER NOT NULL,
                output_chars INTEGER NOT NULL,
                saved_chars INTEGER GENERATED ALWAYS AS (input_chars - output_chars) STORED,
                saved_percent REAL GENERATED ALWAYS AS (
                    CASE WHEN input_chars > 0
                    THEN (CAST(input_chars - output_chars AS REAL) / input_chars * 100)
                    ELSE 0 END
                ) STORED,
                exec_time_ms INTEGER NOT NULL,
                filter_name TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_commands_timestamp ON commands(timestamp);
            CREATE INDEX IF NOT EXISTS idx_commands_filter ON commands(filter_name);

            CREATE TABLE IF NOT EXISTS daily_stats (
                date DATE PRIMARY KEY,
                total_commands INTEGER NOT NULL,
                total_input INTEGER NOT NULL,
                total_output INTEGER NOT NULL,
                total_saved INTEGER GENERATED ALWAYS AS (total_input - total_output) STORED
            );
            "#,
        )?;

        Ok(())
    }

    /// Apply forward-only schema migrations. Each migration is idempotent so
    /// it's safe to run on every open. Old rows have NULL token columns;
    /// queries use COALESCE / IS NOT NULL to handle the mixed state.
    fn migrate(&self) -> Result<()> {
        // v0.7.x → v0.8.0: real-tokenizer columns.
        // We can't use ALTER TABLE ... IF NOT EXISTS in older SQLite, so probe pragma_table_info first.
        let has_tokens_input: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM pragma_table_info('commands') WHERE name = 'tokens_input'",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if !has_tokens_input {
            self.conn.execute_batch(
                r#"
                ALTER TABLE commands ADD COLUMN tokens_input INTEGER;
                ALTER TABLE commands ADD COLUMN tokens_output INTEGER;
                ALTER TABLE commands ADD COLUMN tokenizer_kind TEXT;
                "#,
            )?;
        }

        Ok(())
    }

    /// Track a command execution. `tokens_input` / `tokens_output` are
    /// `Some(n)` only when a real tokenizer (e.g. cl100k) ran; pass `None`
    /// to record byte counts only. `tokenizer_kind` records which method
    /// produced the token numbers (e.g. "cl100k", or "bytes" when none).
    pub fn track_command(
        &self,
        command: &str,
        input_chars: usize,
        output_chars: usize,
        exec_time_ms: u64,
        filter_name: &str,
        tokens_input: Option<usize>,
        tokens_output: Option<usize>,
        tokenizer_kind: &str,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO commands
                (command, input_chars, output_chars, exec_time_ms, filter_name,
                 tokens_input, tokens_output, tokenizer_kind)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                command,
                input_chars as i64,
                output_chars as i64,
                exec_time_ms as i64,
                filter_name,
                tokens_input.map(|n| n as i64),
                tokens_output.map(|n| n as i64),
                tokenizer_kind,
            ],
        )?;

        Ok(())
    }

    /// Get overall statistics.
    pub fn get_statistics(&self) -> Result<Statistics> {
        // Total stats — char counts are always present, token counts only on
        // rows that ran with a real tokenizer (NULL otherwise).
        let (total_commands, total_input, total_output, tok_input, tok_output, tok_rows):
            (i64, i64, i64, Option<i64>, Option<i64>, i64) = self
            .conn
            .query_row(
                r#"
                SELECT
                    COUNT(*),
                    COALESCE(SUM(input_chars), 0),
                    COALESCE(SUM(output_chars), 0),
                    SUM(tokens_input),
                    SUM(tokens_output),
                    COALESCE(SUM(CASE WHEN tokens_input IS NOT NULL THEN 1 ELSE 0 END), 0)
                FROM commands
                "#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?)),
            )
            .unwrap_or((0, 0, 0, None, None, 0));

        let total_saved = total_input - total_output;
        let percent = if total_input > 0 {
            (total_saved as f64 / total_input as f64) * 100.0
        } else {
            0.0
        };

        let tokens = match (tok_input, tok_output) {
            (Some(ti), Some(to)) if tok_rows > 0 => Some(TokenStats {
                rows_with_tokens: tok_rows as usize,
                total_input: ti as usize,
                total_output: to as usize,
                total_saved: (ti - to).max(0) as usize,
                percent: if ti > 0 {
                    ((ti - to) as f64 / ti as f64) * 100.0
                } else {
                    0.0
                },
            }),
            _ => None,
        };

        // By command stats
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                command,
                COUNT(*) as count,
                SUM(input_chars) as input,
                SUM(output_chars) as output,
                SUM(saved_chars) as saved,
                AVG(saved_percent) as percent
            FROM commands
            GROUP BY command
            ORDER BY saved DESC
            LIMIT 20
            "#,
        )?;

        let by_command = stmt
            .query_map([], |row| {
                Ok(CommandStats {
                    command: row.get(0)?,
                    count: row.get(1)?,
                    input_chars: row.get::<_, i64>(2)? as usize,
                    output_chars: row.get::<_, i64>(3)? as usize,
                    saved_chars: row.get::<_, i64>(4)? as usize,
                    percent: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(Statistics {
            total_commands: total_commands as usize,
            total_input: total_input as usize,
            total_output: total_output as usize,
            total_saved: total_saved as usize,
            percent,
            by_command,
            tokens,
        })
    }

    /// Get daily statistics for the last N days.
    pub fn get_daily_stats(&self, days: u32) -> Result<Vec<DailyStats>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                DATE(timestamp) as date,
                COUNT(*) as commands,
                COALESCE(SUM(input_chars), 0) as input,
                COALESCE(SUM(output_chars), 0) as output
            FROM commands
            WHERE timestamp >= DATE('now', ?1)
            GROUP BY DATE(timestamp)
            ORDER BY date ASC
            "#,
        )?;

        let offset = format!("-{} days", days);
        let stats = stmt
            .query_map([&offset], |row| {
                let input: i64 = row.get(2)?;
                let output: i64 = row.get(3)?;
                let saved = input - output;
                let percent = if input > 0 {
                    (saved as f64 / input as f64) * 100.0
                } else {
                    0.0
                };
                Ok(DailyStats {
                    date: row.get(0)?,
                    commands: row.get(1)?,
                    input_chars: input as usize,
                    output_chars: output as usize,
                    saved_chars: saved as usize,
                    percent,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(stats)
    }

    /// Get recent command history.
    pub fn get_history(&self, limit: usize) -> Result<Vec<CommandHistory>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                timestamp,
                command,
                input_chars,
                output_chars,
                saved_percent,
                filter_name
            FROM commands
            ORDER BY timestamp DESC
            LIMIT ?1
            "#,
        )?;

        let history = stmt
            .query_map([limit as i64], |row| {
                Ok(CommandHistory {
                    timestamp: row.get(0)?,
                    command: row.get(1)?,
                    input_chars: row.get::<_, i64>(2)? as usize,
                    output_chars: row.get::<_, i64>(3)? as usize,
                    percent: row.get(4)?,
                    filter_name: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(history)
    }

    /// Get command history filtered by time period.
    pub fn get_history_with_period(&self, limit: usize, days: u32) -> Result<Vec<CommandHistory>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                timestamp,
                command,
                input_chars,
                output_chars,
                saved_percent,
                filter_name
            FROM commands
            WHERE timestamp >= DATE('now', ?1)
            ORDER BY timestamp DESC
            LIMIT ?2
            "#,
        )?;

        let offset = format!("-{} days", days);
        let history = stmt
            .query_map(params![&offset, limit as i64], |row| {
                Ok(CommandHistory {
                    timestamp: row.get(0)?,
                    command: row.get(1)?,
                    input_chars: row.get::<_, i64>(2)? as usize,
                    output_chars: row.get::<_, i64>(3)? as usize,
                    percent: row.get(4)?,
                    filter_name: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(history)
    }
}

fn get_db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_local_dir().context("Could not find local app data directory")?;
    Ok(data_dir.join("wtk").join("tracking.db"))
}

/// Overall statistics.
#[derive(Debug, Serialize)]
pub struct Statistics {
    pub total_commands: usize,
    pub total_input: usize,
    pub total_output: usize,
    pub total_saved: usize,
    pub percent: f64,
    pub by_command: Vec<CommandStats>,
    /// Real-tokenizer aggregates, populated only when at least one row in
    /// the DB has a non-NULL `tokens_input` (i.e. the user has run with the
    /// real tokenizer at least once).
    pub tokens: Option<TokenStats>,
}

/// Aggregated token counts from rows that ran with a real tokenizer.
#[derive(Debug, Serialize)]
pub struct TokenStats {
    /// How many tracked rows have token counts (vs total rows in `Statistics`).
    pub rows_with_tokens: usize,
    pub total_input: usize,
    pub total_output: usize,
    pub total_saved: usize,
    pub percent: f64,
}

/// Per-command statistics.
#[derive(Debug, Serialize)]
pub struct CommandStats {
    pub command: String,
    pub count: usize,
    pub input_chars: usize,
    pub output_chars: usize,
    pub saved_chars: usize,
    pub percent: f64,
}

/// Daily statistics.
#[derive(Debug, Serialize)]
pub struct DailyStats {
    pub date: String,
    pub commands: usize,
    pub input_chars: usize,
    pub output_chars: usize,
    pub saved_chars: usize,
    pub percent: f64,
}

/// Command history entry.
#[derive(Debug, Serialize)]
pub struct CommandHistory {
    pub timestamp: String,
    pub command: String,
    pub input_chars: usize,
    pub output_chars: usize,
    pub percent: f64,
    pub filter_name: String,
}
