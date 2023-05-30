use crate::command::on_receive_command;
use crate::db::DB;
use crate::utils::*;
use db::User;
use dotenv::dotenv;
use log::LevelFilter;

use std::sync::{Arc, Mutex};
use teloxide::{prelude::*, Bot};
use tokio_interval::{clear_timer, set_interval};

mod command;
mod db;
mod gpt;
mod utils;

async fn on_receive_message(state_users: Vec<User>, bot: Bot, msg: Message) {
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

async fn proccess_message(user: User, bot: Bot, msg: Message) {
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

    let bot_token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set.");
    let bot = Bot::new(bot_token);

    let state = Arc::new(Mutex::new(State {
        users: Mutex::new(Vec::new()),
    }));

    let users_list = db.get_users().unwrap();
    state.lock().unwrap().users = Mutex::new(users_list);

    let handler = Update::filter_message().endpoint(
        |bot: Bot, state: Arc<Mutex<State>>, msg: Message| async move {
            let cloned_users = state.lock().unwrap().users.lock().unwrap().clone();

            if is_command_message(msg.clone()) {
                on_receive_command(cloned_users, bot, msg, state).await;
            } else {
                on_receive_message(cloned_users, bot, msg).await;
            }

            respond(())
        },
    );

    let cloned_state = Arc::clone(&state);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![cloned_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
