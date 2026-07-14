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

    conn.execute(
        "CREATE TABLE IF NOT EXISTS scan_history (
            id INTEGER PRIMARY KEY,
            scan_date TEXT NOT NULL,
            todo_count INTEGER NOT NULL
        )",
        [],
    )?;

    // Pas de fausses données en production

    Ok(Arc::new(Mutex::new(conn)))
}

pub fn get_all_skills(db: &DbState) -> Result<Vec<Skill>> {
    let conn = db.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;
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

pub fn delete_skill(db: &DbState, id: i64) -> Result<()> {
    let conn = db.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;
    conn.execute("DELETE FROM installed_skills WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn insert_scan_history(db: &DbState, count: i32) -> Result<()> {
    let conn = db.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;
    let date = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    conn.execute(
        "INSERT INTO scan_history (scan_date, todo_count) VALUES (?1, ?2)",
        params![date, count],
    )?;
    Ok(())
}

pub fn get_scan_history(db: &DbState) -> Result<Vec<(String, i32)>> {
    let conn = db.lock().map_err(|_| rusqlite::Error::InvalidQuery)?;
    let mut stmt = conn.prepare("SELECT scan_date, todo_count FROM scan_history ORDER BY id ASC")?;
    let history_iter = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let mut history = Vec::new();
    for item in history_iter {
        history.push(item?);
    }
    Ok(history)
}
