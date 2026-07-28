use crate::{
    api::{
        message::{get_conversation, send_message},
        user::get_user,
    },
    indexeddb::{self, KeyType, StoredValue},
    message::{decrypt_message, encrypt_message},
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

    let private_key = use_resource(|| async {
        match indexeddb::get(KeyType::Private).await {
            Ok(StoredValue::Bytes(key)) => Ok(key),
            Ok(_) => Err(anyhow::anyhow!("Stored value is not a private key")),
            Err(e) => Err(e),
        }
    });

    let mut conversation = use_resource(move || {
        let sender_id_opt = {
            let guard = user_id.read();
            match &*guard {
                Some(Ok(id)) => Some(id.clone()),
                _ => None,
            }
        };
        let receiver_id_opt = {
            let guard = user.read();
            match &*guard {
                Some(Ok(u)) => Some(u.id.clone()),
                _ => None,
            }
        };

        async move {
            match (sender_id_opt, receiver_id_opt) {
                (Some(s_id), Some(r_id)) => get_conversation(s_id, r_id).await,
                _ => Ok(Vec::new()),
            }
        }
    });

    let decrypted_conversation = use_resource(move || {
        let conv_state: Option<Result<Vec<crate::api::message::Message>, String>> = {
            let guard = conversation.read();
            match &*guard {
                Some(Ok(msgs)) => Some(Ok(msgs.clone())),
                Some(Err(e)) => Some(Err(e.to_string())),
                None => None,
            }
        };

        let key_state: Option<Result<Vec<u8>, String>> = {
            let guard = private_key.read();
            match &*guard {
                Some(Ok(k)) => Some(Ok(k.clone())),
                Some(Err(e)) => Some(Err(e.to_string())),
                None => None,
            }
        };

        async move {
            let messages = match conv_state {
                Some(Ok(m)) => m,
                Some(Err(e)) => return Err(anyhow::anyhow!(e)),
                None => return Ok(Vec::new()),
            };

            let key = match key_state {
                Some(Ok(k)) => k,
                Some(Err(e)) => return Err(anyhow::anyhow!(e)),
                None => return Ok(Vec::new()),
            };

            if messages.is_empty() || key.is_empty() {
                return Ok(Vec::new());
            }

            let mut decrypted = Vec::new();
            for msg in messages {
                let text = decrypt_message(&msg.ciphertext, &key)
                    .await
                    .map_err(|e| anyhow::anyhow!("{:?}", e))?;
                decrypted.push((msg, text));
            }

            Ok::<_, anyhow::Error>(decrypted)
        }
    });

    let receiver = {
        let guard = user.read();
        match &*guard {
            Some(Ok(u)) => u.clone(),
            Some(Err(e)) => {
                web_sys::console::error_1(&e.to_string().into());
                return rsx! { p { "Failed to load receiver." } };
            }
            None => {
                return rsx! { p { "Loading receiver..." } };
            }
        }
    };

    let sender_id = {
        let guard = user_id.read();
        match &*guard {
            Some(Ok(id)) => id.clone(),
            Some(Err(e)) => {
                web_sys::console::error_1(&e.to_string().into());
                return rsx! { p { "Failed to load your user id." } };
            }
            None => {
                return rsx! { p { "Loading..." } };
            }
        }
    };

    let receiver_id = receiver.id;
    let receiver_name = receiver.username.clone();
    let public_key = receiver.public_key.clone();

    rsx! {
        div {
            h2 { "Chat with {receiver_name}" }

            div {
                match &*decrypted_conversation.read() {
                    Some(Ok(messages)) => rsx! {
                        if messages.is_empty() {
                            p { "No messages yet." }
                        } else {
                            for (msg, text) in messages {
                                if msg.sender_id == sender_id {
                                    div {
                                        strong { "You" }
                                        br {}
                                        p { "{text}" }
                                    }
                                } else {
                                    div {
                                        strong { "{receiver_name}" }
                                        br {}
                                        p { "{text}" }
                                    }
                                }
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        p { "Failed to load conversation: {e}" }
                    },
                    None => rsx! {
                        p { "Loading conversation..." }
                    },
                }
            }

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
                    let msg_text = message();
                    if msg_text.is_empty() { return; }

                    let key = public_key.clone();
                    let mut conversation_handle = conversation;
                    let mut message_handle = message;

                    let s_id = sender_id.clone();
                    let r_id = receiver_id.clone();

                    spawn(async move {
                        match encrypt_message(&msg_text, &key).await {
                            Ok(ciphertext) => {
                                match send_message(s_id, r_id, ciphertext).await {
                                    Ok(response) => {
                                        web_sys::console::log_1(
                                            &format!("{:#?}", response).into(),
                                        );
                                        message_handle.set(String::new());
                                        conversation_handle.restart();
                                    }
                                    Err(e) => {
                                        web_sys::console::error_1(
                                            &format!("Failed to send message: {e}").into(),
                                        );
                                    }
                                }
                            }
                            Err(_) => {
                                web_sys::console::error_1(
                                    &"Encryption failed".into(),
                                );
                            }
                        }
                    });
                },
                "Send Message"
            }
        }
    }
}
