use actix_web::{http, App, HttpServer};
use crate::routes::register::register;
use actix_web::middleware::Logger;
use env_logger::Env;
use actix_cors::Cors;

pub mod routes;
pub mod database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_origin("http://localhost:8080")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![
                        http::header::CONTENT_TYPE,
                        http::header::ACCEPT,
                    ])
                    .max_age(3600),
            )
            .service(register)
    })
        .bind(("127.0.0.1", 6767))?
        .run()
        .await
}