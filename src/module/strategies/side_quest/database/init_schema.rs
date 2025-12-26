use crate::util::database::Database;

impl Database {
    pub(in crate::module) fn _side_quest__init_schema(&self) -> rusqlite::Result<()> {
        let conn = self.app_conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS side_quests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                topic TEXT NOT NULL,
                urgency TEXT NOT NULL,
                trigger_date TEXT,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                is_active INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;

        Ok(())
    }
}
