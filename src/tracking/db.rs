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

    /// Track a command execution.
    pub fn track_command(
        &self,
        command: &str,
        input_chars: usize,
        output_chars: usize,
        exec_time_ms: u64,
        filter_name: &str,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO commands (command, input_chars, output_chars, exec_time_ms, filter_name)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![command, input_chars as i64, output_chars as i64, exec_time_ms as i64, filter_name],
        )?;

        Ok(())
    }

    /// Get overall statistics.
    pub fn get_statistics(&self) -> Result<Statistics> {
        // Total stats
        let (total_commands, total_input, total_output): (i64, i64, i64) = self
            .conn
            .query_row(
                r#"
                SELECT
                    COUNT(*),
                    COALESCE(SUM(input_chars), 0),
                    COALESCE(SUM(output_chars), 0)
                FROM commands
                "#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap_or((0, 0, 0));

        let total_saved = total_input - total_output;
        let percent = if total_input > 0 {
            (total_saved as f64 / total_input as f64) * 100.0
        } else {
            0.0
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
        })
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
