use crate::command::on_receive_command;
use crate::db::DB;
use crate::gpt::MyGPT;
use crate::utils::{
    find_user_by_username, is_command_message, send_message, send_tts_multi_parts, send_typing_action,
    send_voice_recording_action, State,
};

use chatgpt::types::Role;
use db::User;
use dotenv::dotenv;
use log::LevelFilter;
use std::error::Error;
use teloxide::prelude::*;
use tokio_interval::{clear_timer, set_interval};

mod command;
mod db;
mod gpt;
mod utils;

lazy_static::lazy_static! {
    static ref GPT: MyGPT = {
        let api_key = std::env::var("GPT_KEY").expect("GPT_KEY must be set.");
        MyGPT::new(&api_key)
    };
}

async fn on_receive_message(state: State, bot: Bot, msg: Message) {
    let user_request = find_user_by_username(&state, msg.chat.username().unwrap());

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

fn is_tts_enabled(user: &User) -> bool {
    let tts_path = std::env::var("TTS_PATH").unwrap_or_default();
    if tts_path.is_empty() || !user.is_voice {
        return false;
    }

    return true;
}

async fn proccess_message(user: User, bot: Bot, msg: Message) {
    let db = DB::new();

    let cloned_user = user.clone();
    let message = msg.text().unwrap();
    let result = GPT.send_msg(msg.chat.id, user, &message).await;

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

fn init_sentry() {
    let _guard = sentry::init((
        dbg!(std::env::var("SENTRY_DSN").unwrap_or_default()),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    std::env::set_var("RUST_BACKTRACE", "1");
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    let db = DB::new();

    log::info!("Starting...");

    init_sentry();

    db.history_migration().await;
    db.users_migration().await;

    let users_list = db.get_users().unwrap();
    let state = State { users: users_list };

    let bot_token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set.");
    let bot = Bot::new(bot_token);

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let cloned_state = state.clone();
        let fut = async move {
            if is_command_message(msg.clone()) {
                tokio::spawn(on_receive_command(cloned_state, bot, msg));
            } else {
                tokio::spawn(on_receive_message(cloned_state, bot, msg));
            }

            Ok(())
        };
        async move { fut.await }
    })
    .await;
}
