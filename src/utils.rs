use std::error::Error;

use crate::db::User;
use reqwest;
use teloxide::{prelude::*, types::ChatAction, types::InputFile};

#[derive(Clone, Debug)]
pub struct State {
    pub users: Vec<User>,
}

pub fn find_user_by_username<'a>(state: &'a State, username: &'a str) -> Option<&'a User> {
    state.users.iter().find(|user| user.user_name == username)
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

    let resp = response.unwrap();

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
    let message = msg.text().unwrap();
    let first_char = message.chars().nth(0).unwrap();
    if first_char == '/' {
        return true;
    }
    return false;
}
