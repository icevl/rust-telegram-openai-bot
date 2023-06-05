use crate::{
    db::{User, DB},
    gpt::MyGPT,
};
use chatgpt::types::Role;
use reqwest;
use std::{env, error::Error, fs, sync::Mutex};
use teloxide::{
    net::Download,
    prelude::*,
    types::InputFile,
    types::{ChatAction, FileMeta},
};
use tokio::fs::OpenOptions;
use tokio_interval::{clear_timer, set_interval};
use uuid::Uuid;

#[derive(Debug)]
pub struct State {
    pub users: Mutex<Vec<User>>,
}

pub struct TextMessage<'a> {
    pub user: User,
    pub bot: Bot,
    pub chat_id: ChatId,
    pub message: &'a str,
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
    message: &str,
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

pub async fn send_tts_multi_parts(bot: Bot, chat_id: ChatId, message: &str) {
    let parts = textwrap::wrap(message, 800);

    for (_, part) in parts.iter().enumerate() {
        let cloned_bot = bot.clone();
        let tts_success = send_tts(cloned_bot, chat_id, &part.to_string()).await;
        match tts_success {
            Ok(_) => {}
            Err(error) => {
                sentry::capture_error(&*error);
                send_message(bot.clone(), chat_id, &part).await;
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

pub async fn proccess_text_message(args: TextMessage<'_>) {
    let gpt_api_key = std::env::var("GPT_KEY").expect("GPT_KEY must be set.");
    let db = DB::new();
    let gpt = MyGPT::new(&gpt_api_key);
    let cloned_user = args.user.clone();

    let result = gpt.send_msg(args.chat_id, args.user, &args.message).await;

    log::info!("[{}]: {}", cloned_user.user_name, args.message);

    match result {
        Ok(content) => {
            log::info!("[bot]: {}", content);
            let is_voice_response =
                is_tts_enabled(&cloned_user) && !is_code_listing(content.as_str());

            db.save_message(args.chat_id, Role::Assistant, &content);

            if !is_voice_response {
                send_message(args.bot, args.chat_id, &content).await;
                return;
            }

            send_tts_multi_parts(args.bot.clone(), args.chat_id, &content).await;
        }
        Err(error) => {
            send_message(args.bot, args.chat_id, "I broke down. I feel bad").await;

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

pub async fn asr(bot: Bot, file: &FileMeta) -> &'static str {
    let dir = env::temp_dir();
    let tmp_file_path = format!("{}{}.ogg", dir.display(), Uuid::new_v4());

    log::info!("Voice: {:?}", file.id);
    log::info!("Temporary directory: {}", tmp_file_path);

    let mut local_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(&tmp_file_path)
        .await
        .unwrap();

    let file_response = bot.get_file(&file.id).await;
    match file_response {
        Ok(file_request) => {
            log::info!("FILE: {:?}", file_request.path);
            bot.download_file(&file_request.path, &mut local_file)
                .await
                .unwrap();

            // TODO: proccess data to STT API

            fs::remove_file(tmp_file_path).unwrap();

            "test from asr goes here"
        }
        Err(err) => {
            log::error!("Failed to create file request: {:?}", err);
            ""
        }
    }
}

pub fn is_code_listing(text: &str) -> bool {
    let indented_lines = text.lines().filter(|line| line.starts_with("    ")).count();
    let total_lines = text.lines().count();
    let indentation_ratio = indented_lines as f32 / total_lines as f32;

    if indentation_ratio > 0.5 {
        return true;
    }

    let keywords = ["fn"];
    if keywords.iter().any(|&keyword| text.contains(keyword)) {
        return true;
    }

    let syntax_characters = ['{', '}', '(', ')', '`'];
    if syntax_characters.iter().any(|&c| text.contains(c)) {
        return true;
    }

    false
}

pub async fn proccess_message(user: User, bot: Bot, msg: Message) {
    let voice = msg.voice();
    let message = msg.text();

    let mut content = "";

    if voice.is_some() {
        content = asr(bot.clone(), &voice.unwrap().file).await;
    }

    if message.is_some() {
        content = msg.text().unwrap();
    }

    if content.trim().is_empty() {
        return;
    }

    proccess_text_message(TextMessage {
        user: user,
        bot: bot,
        chat_id: msg.chat.id,
        message: content.clone(),
    })
    .await;
}

pub async fn on_receive_message(state_users: Vec<User>, bot: Bot, msg: Message) {
    let user_request = find_user_by_username(&state_users, msg.chat.username().unwrap());
    let bot_cloned = bot.clone();

    if let Some(user) = user_request {
        let is_voice_response_required = is_tts_enabled(&user);

        let typing_interval = set_interval!(
            move || {
                if is_voice_response_required {
                    tokio::spawn(send_voice_recording_action(bot_cloned.clone(), msg.chat.id));
                } else {
                    tokio::spawn(send_typing_action(bot_cloned.clone(), msg.chat.id));
                }
            },
            3000
        );

        proccess_message(user.clone(), bot, msg).await;
        clear_timer!(typing_interval)
    } else {
        send_message(bot, msg.chat.id, "Access denied").await;
    }
}
