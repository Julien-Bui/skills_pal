use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use crate::models::Skill;

pub type DbState = Arc<Mutex<Connection>>;

pub fn init_db(db_path: &str) -> Result<DbState> {
    let conn = Connection::open(db_path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS installed_skills (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            github_url TEXT NOT NULL,
            is_active INTEGER NOT NULL
        )",
        [],
    )?;

    // Insertion de fausses données pour la démonstration finale
    let count: i32 = conn.query_row("SELECT COUNT(*) FROM installed_skills", [], |row| row.get(0))?;
    if count == 0 {
        conn.execute("INSERT INTO installed_skills (name, description, github_url, is_active) VALUES (?1, ?2, ?3, ?4)", params!["Rust Analyzer", "Scans for memory leaks.", "https://github.com/rust-analyzer", 1])?;
        conn.execute("INSERT INTO installed_skills (name, description, github_url, is_active) VALUES (?1, ?2, ?3, ?4)", params!["Security Scanner", "Detects vulnerabilities.", "https://github.com/sec", 1])?;
        conn.execute("INSERT INTO installed_skills (name, description, github_url, is_active) VALUES (?1, ?2, ?3, ?4)", params!["Code Linter", "Enforces formatting rules.", "https://github.com/linter", 0])?;
    }

    Ok(Arc::new(Mutex::new(conn)))
}

pub fn get_all_skills(db: &DbState) -> Result<Vec<Skill>> {
    let conn = db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, description, github_url, is_active FROM installed_skills")?;
    let skill_iter = stmt.query_map([], |row| {
        Ok(Skill {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            description: row.get(2)?,
            github_url: row.get(3)?,
            is_active: row.get::<_, i32>(4)? == 1,
        })
    })?;

    let mut skills = Vec::new();
    for skill in skill_iter {
        skills.push(skill?);
    }
    Ok(skills)
}

pub fn add_skill() { }
pub fn delete_skill(db: &DbState, id: i64) -> Result<()> {
    let conn = db.lock().unwrap();
    conn.execute("DELETE FROM installed_skills WHERE id = ?1", params![id])?;
    Ok(())
}
