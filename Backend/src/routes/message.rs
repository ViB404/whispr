use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub sender_id: i64,
    pub receiver_id: i64,
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct SendMessageResponse {
    pub id: i64,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    pub ciphertext: Vec<u8>,
    pub created_at: i64,
}

#[post("/messages")]
pub async fn send_message(payload: web::Json<SendMessageRequest>) -> impl Responder {
    let conn = match connect_database().await {
        Ok(conn) => conn,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    let message = NewMessage {
        sender_id: payload.sender_id,
        receiver_id: payload.receiver_id,
        ciphertext: payload.ciphertext.clone(),
    };

    match save_message(&conn, &message) {
        Ok(id) => HttpResponse::Created().json(SendMessageResponse {
            id,
            message: "Message sent successfully".into(),
        }),

        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

use actix_web::{get, post, web, HttpResponse, Responder};

use crate::database::{
    connect_database, get_conversation, get_message_by_id, save_message, NewMessage,
};

#[get("/messages/{id}")]
pub async fn get_message(path: web::Path<i64>) -> impl Responder {
    let conn = match connect_database().await {
        Ok(conn) => conn,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    match get_message_by_id(&conn, path.into_inner()) {
        Ok(message) => HttpResponse::Ok().json(MessageResponse {
            id: message.id,
            sender_id: message.sender_id,
            receiver_id: message.receiver_id,
            ciphertext: message.ciphertext,
            created_at: message.created_at,
        }),

        Err(_) => HttpResponse::NotFound().body("Message not found"),
    }
}

#[derive(Deserialize)]
pub struct ConversationPath {
    pub user1: i64,
    pub user2: i64,
}

#[get("/messages/{user1}/{user2}")]
pub async fn get_users_conversation(path: web::Path<ConversationPath>) -> impl Responder {
    let conn = match connect_database().await {
        Ok(conn) => conn,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    let path = path.into_inner();

    match get_conversation(&conn, path.user1, path.user2) {
        Ok(messages) => HttpResponse::Ok().json(messages),

        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
