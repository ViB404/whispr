use anyhow::Error;
use rusqlite::{Connection, params};
use serde::Serialize;

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

    CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        sender_id INTEGER NOT NULL,
        receiver_id INTEGER NOT NULL,
        ciphertext BLOB NOT NULL,
        created_at INTEGER NOT NULL DEFAULT (unixepoch('subsec') * 1000)
    );
    ",
    )?;

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    pub ciphertext: Vec<u8>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct NewMessage {
    pub sender_id: i64,
    pub receiver_id: i64,
    pub ciphertext: Vec<u8>,
}

pub fn save_message(conn: &Connection, message: &NewMessage) -> Result<i64, Error> {
    conn.execute(
        "
        INSERT INTO messages (
            sender_id,
            receiver_id,
            ciphertext
        ) VALUES (
            ?1,
            ?2,
            ?3
        )
        ",
        params![message.sender_id, message.receiver_id, message.ciphertext,],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_message_by_id(conn: &Connection, message_id: i64) -> Result<Message, Error> {
    let message = conn.query_row(
        "
        SELECT
            id,
            sender_id,
            receiver_id,
            ciphertext,
            created_at
        FROM messages
        WHERE id = ?1
        ",
        params![message_id],
        |row| {
            Ok(Message {
                id: row.get(0)?,
                sender_id: row.get(1)?,
                receiver_id: row.get(2)?,
                ciphertext: row.get(3)?,
                created_at: row.get(4)?,
            })
        },
    )?;

    Ok(message)
}

pub fn get_ciphertext_by_message_id(conn: &Connection, message_id: i64) -> Result<Vec<u8>, Error> {
    let ciphertext = conn.query_row(
        "
        SELECT ciphertext
        FROM messages
        WHERE id = ?1
        ",
        params![message_id],
        |row| row.get(0),
    )?;

    Ok(ciphertext)
}

pub fn get_received_messages(conn: &Connection, receiver_id: i64) -> Result<Vec<Message>, Error> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            sender_id,
            receiver_id,
            ciphertext,
            created_at
        FROM messages
        WHERE receiver_id = ?1
        ORDER BY created_at DESC
        ",
    )?;

    let rows = stmt.query_map(params![receiver_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            sender_id: row.get(1)?,
            receiver_id: row.get(2)?,
            ciphertext: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut messages = Vec::new();

    for row in rows {
        messages.push(row?);
    }

    Ok(messages)
}

pub fn get_conversation(conn: &Connection, user1: i64, user2: i64) -> Result<Vec<Message>, Error> {
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            sender_id,
            receiver_id,
            ciphertext,
            created_at
        FROM messages
        WHERE
            (sender_id = ?1 AND receiver_id = ?2)
            OR
            (sender_id = ?2 AND receiver_id = ?1)
        ORDER BY created_at ASC
        ",
    )?;

    let rows = stmt.query_map(params![user1, user2], |row| {
        Ok(Message {
            id: row.get(0)?,
            sender_id: row.get(1)?,
            receiver_id: row.get(2)?,
            ciphertext: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut messages = Vec::new();

    for row in rows {
        messages.push(row?);
    }

    Ok(messages)
}

#[derive(Debug, Clone, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub public_key: Vec<u8>,
}

pub fn get_user_by_username(conn: &Connection, username: &str) -> Result<User, Error> {
    let user = conn.query_row(
        "
        SELECT
            id,
            username,
            public_key
        FROM users
        WHERE username = ?1
        ",
        params![username],
        |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                public_key: row.get(2)?,
            })
        },
    )?;

    Ok(user)
}
