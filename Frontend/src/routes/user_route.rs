use crate::{
    api::message::send_message,
    api::user::get_user,
    indexeddb::{self, KeyType, StoredValue},
    message::encrypt_message,
};

use dioxus::prelude::*;

#[component]
pub fn SendMessagePage(username: String) -> Element {
    let mut message = use_signal(String::new);

    let user = use_resource({
        let username = username.clone();
        move || {
            let username = username.clone();
            async move { get_user(&username).await }
        }
    });

    let user_id = use_resource(|| async {
        match indexeddb::get(KeyType::UserId).await {
            Ok(StoredValue::UserId(id)) => Ok(id),
            Ok(_) => Err(anyhow::anyhow!("Stored value is not a user id")),
            Err(e) => Err(e),
        }
    });

    let user_guard = user.read_unchecked();

    let receiver = match &*user_guard {
        Some(Ok(user)) => user,
        Some(Err(e)) => {
            web_sys::console::error_1(&e.to_string().into());
            return rsx! { "Failed to load receiver." };
        }
        None => {
            return rsx! { "Loading..." };
        }
    };

    let sender_id = match &*user_id.read_unchecked() {
        Some(Ok(id)) => *id,
        Some(Err(e)) => {
            web_sys::console::error_1(&e.to_string().into());
            return rsx! { "Failed to load your user id." };
        }
        None => {
            return rsx! { "Loading..." };
        }
    };

    let receiver_id = receiver.id;
    let public_key = receiver.public_key.clone();

    rsx! {
        input {
            r#type: "text",
            placeholder: "Enter message",
            value: "{message}",
            oninput: move |e| {
                message.set(e.value());
            },
        }

        button {
            onclick: move |_| {
                let msg = message();
                let key = public_key.clone();

                spawn(async move {
                    match encrypt_message(&msg, &key).await {
                        Ok(ciphertext) => {
                            match send_message(
                                sender_id,
                                receiver_id,
                                ciphertext,
                            ).await {
                                Ok(response) => {
                                    web_sys::console::log_1(
                                        &format!("{:#?}", response).into(),
                                    );
                                }
                                Err(e) => {
                                    web_sys::console::error_1(
                                        &format!("Failed to send message: {e}").into(),
                                    );
                                }
                            }
                        }

                        Err(_e) => {;
                        }
                    }
                });
            },

            "Send Message"
        }
    }
}
