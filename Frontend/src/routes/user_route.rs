use crate::{
    api::message::send_message,
    api::user::get_user,
};

use dioxus::prelude::*;
use crate::message::encrypt_message;

#[component]
pub fn SendMessage(id: String) -> Element {
    let mut message = use_signal(String::new);

    let user = use_resource({
        let id = id.clone();

        move || {
            let id = id.clone();

            async move { get_user(&id).await }
        }
    });

    rsx! {
        match &*user.read_unchecked() {
            Some(Ok(user)) => {
                let receiver_id = user.id;
                let public_key = user.public_key.clone();

                rsx! {
                    input {
                        r#type: "text",
                        placeholder: "Enter message",
                        value: "{message}",
                        oninput: move |e| message.set(e.value()),
                    }

                    button {
                        onclick: move |_| {
                            let msg = message();
                            let key = public_key.clone();

                            spawn(async move {
                                let ciphertext = encrypt_message(&msg, &key).await.expect("can't encrypt the message");

                                let response = send_message(
                                    /* sender_id (TODO: fix it later) */ 1,
                                    receiver_id,
                                    ciphertext,
                                )
                                .await
                                .unwrap();

                                web_sys::console::log_1(
                                    &format!("{:#?}", response).into(),
                                );
                            });
                        },

                        "Send Message"
                    }
                }
            }

            Some(Err(e)) => {
                web_sys::console::error_1(&e.to_string().into());

                rsx! {
                    "Failed to load"
                }
            }

            None => rsx! {
                "Loading..."
            },
        }
    }
}