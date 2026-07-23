use actix_web::{App, HttpServer};
use crate::routes::register::register;

pub mod routes;
pub mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(register)
    })
        .bind(("127.0.0.1", 6767))?
        .run()
        .await
}