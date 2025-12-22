use crate::util::database::Database;

impl Database {
    pub(super) fn _clear_knowledge(&self) -> rusqlite::Result<()> {
        let conn = self.knowledge_conn.lock().unwrap();
        conn.execute("DELETE FROM knowledge", [])?;
        Ok(())
    }
}
