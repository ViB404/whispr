mod api;
mod crypto;
mod indexeddb;
mod message;
pub mod routes;
mod types;
pub mod user;

use crate::routes::user_route::SendMessagePage;
use crate::user::register;
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;

fn main() {
    launch(App);
}

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/message")]
    Message {},
    #[route("/user/:username")]
    SendMessagePage { username: String },
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    let mut username = use_signal(String::new);
    let nav = use_navigator();

    rsx! {
        div {
            style: "
                min-height: 100vh;
                display: flex;
                align-items: center;
                justify-content: center;
                background: #f5f5f5;
                font-family: sans-serif;
            ",

            div {
                style: "
                    width: 360px;
                    padding: 32px;
                    background: white;
                    border-radius: 12px;
                    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
                ",

                h1 {
                    style: "
                        margin: 0 0 8px 0;
                        font-size: 28px;
                        text-align: center;
                    ",
                    "Sign In"
                }

                p {
                    style: "
                        margin: 0 0 24px 0;
                        color: #666;
                        text-align: center;
                        font-size: 14px;
                    ",
                    "Enter your username to continue"
                }

                input {
                    style: "
                        width: 100%;
                        box-sizing: border-box;
                        padding: 12px;
                        border: 1px solid #ddd;
                        border-radius: 8px;
                        outline: none;
                        font-size: 15px;
                        margin-bottom: 16px;
                    ",
                    placeholder: "Username",
                    value: "{username}",
                    oninput: move |e| username.set(e.value()),
                }

                button {
                    style: "
                        width: 100%;
                        padding: 12px;
                        border: none;
                        border-radius: 8px;
                        background: #111;
                        color: white;
                        font-size: 15px;
                        cursor: pointer;
                    ",

                    onclick: move |_| {
                        let user = username();

                        spawn(async move {
                            match register(user).await {
                                Ok(_) => {
                                    nav.push(Route::Message {});
                                }
                                Err(err) => {
                                    eprintln!("Registration failed: {err:?}");
                                }
                            }
                        });
                    },

                    "Sign In"
                }
            }
        }
    }
}