mod api;
mod crypto;
mod indexeddb;
mod message;
pub mod routes;
mod types;
pub mod user;

use crate::api::message::send_message;
use crate::message::{decrypt_message, encrypt_message};
use crate::routes::user_route::SendMessagePage;
use crate::user::register;
use dioxus::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

fn main() {
    launch(App);
}

#[derive(Routable, Clone)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/encrypt")]
    Encrypt {},
    #[route("/decrypt")]
    Decrypt {},
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

    rsx! {
        h1 { "Generate Keys 🔑" }

        input {
            placeholder: "Username",
            value: "{username}",
            oninput: move |e| username.set(e.value()),
        }

        button {
            onclick: move |_| {
                let username = username();

                spawn_local(async move {
                    if let Err(_err) = register(username).await {
                        eprintln!("Something went wrong!");
                    }
                });
            },

            "Generate"
        }

        br {}

        Link { to: Route::Encrypt {}, "Encrypt" }
        br {}
        Link { to: Route::Decrypt {}, "Decrypt" }
    }
}

#[component]
fn Encrypt() -> Element {
    let mut message = use_signal(String::new);
    let mut output = use_signal(String::new);

    rsx! {
                h1 { "Encrypt" }

                input {
                    placeholder: "Message",
                    value: "{message}",
                    oninput: move |e| message.set(e.value()),
                }

                button {
                    onclick: move |_| {
            let msg = message();

            spawn_local(async move {
                let key = match indexeddb::get(indexeddb::KeyType::Public).await {
        Ok(indexeddb::StoredValue::Bytes(key)) => key,
        Ok(_) => {
            eprintln!("Stored value is not a public key");
            return;
        }
        Err(e) => {
            eprintln!("Failed to load public key: {:#?}", e);
            return;
        }
                };

                if !key.is_empty() {
                    println!(
                        "First 32 bytes: {:02X?}",
                        &key[..std::cmp::min(32, key.len())]
                    );
                }

                match encrypt_message(&msg, &key).await {
                    Ok(encrypted) => {
                        output.set(
                            encrypted
                                .iter()
                                .map(|b| b.to_string())
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                    }

                    Err(err) => {
                        console::error_1(&err);
                    }
                }
            });
        },

                    "Encrypt"
                }

        p { "{output}" }

        Link { to: Route::Home {}, "Home" }
    }
}

#[component]
fn Decrypt() -> Element {
    let mut encrypted = use_signal(String::new);
    let mut output = use_signal(String::new);

    rsx! {
                h1 { "Decrypt" }

                textarea {
                    value: "{encrypted}",
                    oninput: move |e| encrypted.set(e.value()),
                }

                button {
                    onclick: move |_| {
            let text = encrypted();

            spawn_local(async move {

                let key = match indexeddb::get(indexeddb::KeyType::Private).await {
        Ok(indexeddb::StoredValue::Bytes(key)) => key,
        Ok(_) => {
            eprintln!("Stored value is not a private key");
            return;
        }
        Err(e) => {
            eprintln!("Failed to load private key: {:#?}", e);
            return;
        }
    };

                if !key.is_empty() {
                    println!(
                        "First 32 bytes: {:02X?}",
                        &key[..std::cmp::min(32, key.len())]
                    );
                }

                let bytes: Vec<u8> = text
                    .split(',')
                    .filter_map(|v| v.trim().parse::<u8>().ok())
                    .collect();

                match decrypt_message(&bytes, &key).await {
                    Ok(message) => {
                        println!("Decryption successful!");
                        println!("Message: {}", message);
                        output.set(message);
                    }

                    Err(err) => {
                        eprintln!("Decryption failed!");
                        console::error_1(&err);
                    }
                }
            });
        },
                    "Decrypt"
                }
                p { "{output}" }
                Link { to: Route::Home {}, "Home" }
            }
}
