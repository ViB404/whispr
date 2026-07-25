use actix_web::web;

pub mod message;
pub mod user;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(user::register)
        .service(message::send_message)
        .service(message::get_message)
        .service(message::get_users_conversation)
        .service(user::get_user);
}
