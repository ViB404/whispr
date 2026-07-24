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

use actix_web::{HttpResponse, Responder, post, web};

use crate::database::NewUser;
use crate::database::{connect_database, save_user};

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

        Err(err) => {
            HttpResponse::InternalServerError().body(err.to_string())
        }
    }
}
