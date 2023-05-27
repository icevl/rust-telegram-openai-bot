use std::{error::Error, sync::Mutex};

use crate::{
    db::{User, DB},
    gpt::MyGPT,
};
use chatgpt::types::Role;
use reqwest;
use teloxide::{prelude::*, types::ChatAction, types::InputFile};

#[derive(Debug)]
pub struct State {
    pub users: Mutex<Vec<User>>,
}

pub fn find_user_by_username<'a>(users: &'a Vec<User>, username: &'a str) -> Option<&'a User> {
    users.iter().find(|user| user.user_name == username)
}

pub async fn send_message(bot: Bot, chat_id: ChatId, message: &str) {
    let result = bot.send_message(chat_id, message).await;

    match result {
        Ok(_) => {}
        Err(err) => {
            sentry::capture_error(&err);
        }
    }
}

pub async fn send_tts(
    bot: Bot,
    chat_id: ChatId,
    message: &String,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let json_body = serde_json::json!({ "text": message });

    let client = reqwest::Client::new();
    let response = client
        .post(std::env::var("TTS_PATH").unwrap_or_default())
        .json(&json_body)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err("HTTP tts error".into());
            }

            let body = resp.bytes().await;
            let audio_stream = InputFile::memory(body.unwrap());
            let response = bot.send_voice(chat_id, audio_stream).await;

            match response {
                Ok(_) => Ok(true),
                Err(error) => Err(error.into()),
            }
        }
        Err(error) => {
            sentry::capture_error(&error);
            return Err(error.into());
        }
    }
}

pub async fn send_tts_multi_parts(bot: Bot, chat_id: ChatId, message: &String) {
    let parts = textwrap::wrap(message, 800);

    for (_, part) in parts.iter().enumerate() {
        let cloned_bot = bot.clone();
        let tts_success = send_tts(cloned_bot, chat_id, &part.to_string()).await;
        match tts_success {
            Ok(_) => {}
            Err(error) => {
                sentry::capture_error(&*error);
                send_message(bot.clone(), chat_id, &message).await;
            }
        }
    }
}

pub async fn send_typing_action(bot: Bot, chat_id: ChatId) {
    match bot.send_chat_action(chat_id, ChatAction::Typing).await {
        Ok(_) => {}
        Err(err) => {
            sentry::capture_error(&err);
        }
    };
}

pub async fn send_voice_recording_action(bot: Bot, chat_id: ChatId) {
    match bot.send_chat_action(chat_id, ChatAction::RecordVoice).await {
        Ok(_) => {}
        Err(err) => {
            sentry::capture_error(&err);
        }
    };
}

pub fn is_command_message(msg: Message) -> bool {
    let message = msg.text();

    match message {
        Some(text) => {
            let first_char = text.chars().nth(0).unwrap();
            if first_char == '/' {
                return true;
            }
            return false;
        }
        None => return false,
    }
}

pub async fn proccess_text_message(user: User, bot: Bot, msg: Message) {
    let gpt_api_key = std::env::var("GPT_KEY").expect("GPT_KEY must be set.");
    let db = DB::new();
    let gpt = MyGPT::new(&gpt_api_key);
    let message = msg.text().unwrap();
    let cloned_user = user.clone();

    let result = gpt.send_msg(msg.chat.id, user, &message).await;

    log::info!("[{}]: {}", cloned_user.user_name, message);

    match result {
        Ok(content) => {
            log::info!("[bot]: {}", content);
            let is_voice_response = is_tts_enabled(&cloned_user);

            db.save_message(msg.chat.id, Role::Assistant, content.clone());

            if !is_voice_response {
                send_message(bot, msg.chat.id, &content).await;
                return;
            }

            send_tts_multi_parts(bot.clone(), msg.chat.id, &content).await;
        }
        Err(error) => {
            send_message(bot, msg.chat.id, "I broke down. I feel bad").await;

            let error_ref: &dyn Error = &*error;
            sentry::capture_error(error_ref);
        }
    }
}

pub fn is_tts_enabled(user: &User) -> bool {
    let tts_path = std::env::var("TTS_PATH").unwrap_or_default();
    if tts_path.is_empty() || !user.is_voice {
        return false;
    }

    return true;
}
