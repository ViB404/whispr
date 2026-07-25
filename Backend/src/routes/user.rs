use crate::database::{connect_database, save_user};
use crate::database::{get_user_by_username, NewUser};
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: i64,
    pub message: String,
}

#[post("/register")]
pub async fn register(payload: web::Json<RegisterRequest>) -> impl Responder {
    let conn = match connect_database().await {
        Ok(conn) => conn,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    let user = NewUser {
        username: payload.username.clone(),
        public_key: payload.public_key.clone(),
    };

    match save_user(&conn, &user) {
        Ok(id) => HttpResponse::Created().json(RegisterResponse {
            id,
            message: "User registered successfully".into(),
        }),

        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/users/{username}")]
pub async fn get_user(username: web::Path<String>) -> impl Responder {
    let conn = match connect_database().await {
        Ok(conn) => conn,
        Err(err) => {
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    };

    match get_user_by_username(&conn, &username) {
        Ok(user) => HttpResponse::Ok().json(user),

        Err(_) => HttpResponse::NotFound().body("User not found"),
    }
}
