use std::collections::HashMap;

use crate::{
    api::{
        message::{get_conversation, send_message, Message},
        user::get_user,
    },
    indexeddb::{self, KeyType, SentMessage, StoredValue},
    message::{decrypt_message, encrypt_message},
};

use dioxus::prelude::*;

pub fn merge_messages(
    current_user_id: i64,
    local: Vec<SentMessage>,
    server: Vec<Message>,
) -> Vec<SentMessage> {
    let mut local_map: HashMap<(i64, i64, i64), SentMessage> = local
        .into_iter()
        .map(|m| ((m.sender_id, m.receiver_id, m.timestamp), m))
        .collect();

    let mut merged: Vec<SentMessage> = Vec::with_capacity(server.len());

    for msg in server {
        let timestamp = msg.created_at * 1000;

        if msg.sender_id == current_user_id {
            if let Some(local_msg) = local_map.remove(&(msg.sender_id, msg.receiver_id, timestamp))
            {
                merged.push(local_msg);
            } else {
                merged.push(SentMessage {
                    sender_id: msg.sender_id,
                    receiver_id: msg.receiver_id,
                    message: "Can't be decoded".to_string(),
                    timestamp,
                });
            }
        } else {
            merged.push(SentMessage {
                sender_id: msg.sender_id,
                receiver_id: msg.receiver_id,
                message: "Can't be decoded".to_string(),
                timestamp,
            });
        }
    }

    merged.sort_by_key(|m| m.timestamp);
    merged
}

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
                Some(Ok(id)) => Some(*id),
                _ => None,
            }
        };
        let receiver_id_opt = {
            let guard = user.read();
            match &*guard {
                Some(Ok(u)) => Some(u.id),
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

    let mut local_messages =
        use_resource(|| async { indexeddb::get_messages().await.unwrap_or_default() });

    let processed_conversation = use_resource(move || {
        let conv_state: Option<Result<Vec<Message>, String>> = {
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

        let local_state: Vec<SentMessage> = {
            let guard = local_messages.read();
            match &*guard {
                Some(msgs) => msgs.clone(),
                None => Vec::new(),
            }
        };

        let current_user_id = {
            let guard = user_id.read();
            match &*guard {
                Some(Ok(id)) => Some(*id),
                _ => None,
            }
        };

        async move {
            let server_msgs = match conv_state {
                Some(Ok(m)) => m,
                Some(Err(e)) => return Err(anyhow::anyhow!(e)),
                None => return Ok(Vec::new()),
            };

            let key = match key_state {
                Some(Ok(k)) => k,
                Some(Err(e)) => return Err(anyhow::anyhow!(e)),
                None => return Ok(Vec::new()),
            };

            let c_id = match current_user_id {
                Some(id) => id,
                None => return Ok(Vec::new()),
            };

            if server_msgs.is_empty() || key.is_empty() {
                return Ok(Vec::new());
            }

            let merged = merge_messages(c_id, local_state, server_msgs.clone());
            let mut final_list = Vec::with_capacity(merged.len());

            for mut msg in merged {
                if msg.sender_id != c_id {
                    if let Some(server_msg) = server_msgs.iter().find(|sm| {
                        sm.sender_id == msg.sender_id && sm.created_at * 1000 == msg.timestamp
                    }) {
                        match decrypt_message(&server_msg.ciphertext, &key).await {
                            Ok(text) => msg.message = text,
                            Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Decryption error: {:?}", e).into(),
                                );
                                msg.message = "Failed to decrypt".to_string();
                            }
                        }
                    }
                }
                final_list.push(msg);
            }

            Ok::<_, anyhow::Error>(final_list)
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
            Some(Ok(id)) => *id,
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
                match &*processed_conversation.read() {
                    Some(Ok(messages)) => rsx! {
                        if messages.is_empty() {
                            p { "No messages yet." }
                        } else {
                            for msg in messages {
                                if msg.sender_id == sender_id {
                                    div {
                                        strong { "You" }
                                        br {}
                                        p { "{msg.message}" }
                                    }
                                } else {
                                    div {
                                        strong { "{receiver_name}" }
                                        br {}
                                        p { "{msg.message}" }
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
                    let mut local_messages_handle = local_messages;
                    let mut message_handle = message;

                    let s_id = sender_id;
                    let r_id = receiver_id;

                    spawn(async move {
                        match encrypt_message(&msg_text, &key).await {
                            Ok(ciphertext) => {
                                match send_message(s_id, r_id, ciphertext).await {
                                    Ok(response) => {
                                        web_sys::console::log_1(
                                            &format!("{:#?}", response).into(),
                                        );

                                        if let Err(e) = indexeddb::save_message(SentMessage {
                                            sender_id: s_id,
                                            receiver_id: r_id,
                                            message: msg_text.clone(),
                                            timestamp: js_sys::Date::now() as i64,
                                        })
                                        .await
                                        {
                                            web_sys::console::error_1(
                                                &format!("Failed to cache message: {e}").into(),
                                            );
                                        }

                                        message_handle.set(String::new());
                                        local_messages_handle.restart();
                                        conversation_handle.restart();
                                    }
                                    Err(e) => {
                                        web_sys::console::error_1(
                                            &format!("Failed to send message: {e}").into(),
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                web_sys::console::error_1(
                                    &format!("Encryption failed: {e:?}").into(),
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
