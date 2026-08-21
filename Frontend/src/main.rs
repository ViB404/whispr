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
        h1 { "Sign IN" }

        input {
            placeholder: "Username",
            value: "{username}",
            oninput: move |e| username.set(e.value()),
        }

        button {
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

#[component]
fn Message() -> Element {
    let mut username = use_signal(String::new);
    let navigator = use_navigator();

    rsx! {
        div { "Message User" }

        input {
            placeholder: "Username",
            value: "{username}",
            oninput: move |e| username.set(e.value()),
        }

        button {
            onclick: move |_| {
                let username = username();
                navigator.push(Route::SendMessagePage { username });
            },

            "Message"
        }
    }
}
