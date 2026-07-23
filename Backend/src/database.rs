use anyhow::Error;
use rusqlite::{Connection, params};

pub async fn connect_database() -> Result<Connection, Error> {
    let conn = Connection::open("users.db")?;

    init_database(&conn)?;

    Ok(conn)
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub public_key: Vec<u8>,
}

pub fn save_user(conn: &Connection, user: &NewUser) -> Result<i64, Error> {
    conn.execute(
        "
        INSERT INTO users (
            username,
            public_key
        ) VALUES (
            ?1,
            ?2
        )
        ",
        params![&user.username, &user.public_key,],
    )?;

    Ok(conn.last_insert_rowid())
}

fn init_database(conn: &Connection) -> Result<(), Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            public_key BLOB NOT NULL
        );
        ",
    )?;

    Ok(())
}
