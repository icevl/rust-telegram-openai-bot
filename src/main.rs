use crate::db::DB;
use crate::gpt::MyGPT;
use chatgpt::types::Role;
use db::User;
use dotenv::dotenv;
use log::LevelFilter;
use std::error::Error;
use teloxide::{prelude::*, types::ChatAction};
use tokio_interval::{clear_timer, set_interval};

mod db;
mod gpt;

#[derive(Clone, Debug)]
struct State {
    users: Vec<User>,
}

lazy_static::lazy_static! {
    static ref GPT: MyGPT = {
        let api_key = std::env::var("GPT_KEY").expect("GPT_KEY must be set.");
        MyGPT::new(&api_key)
    };
}

async fn send_typing_action(bot: Bot, chat_id: ChatId) {
    match bot.send_chat_action(chat_id, ChatAction::Typing).await {
        Ok(_) => {}
        Err(err) => {
            sentry::capture_error(&err);
        }
    };
}

async fn send_message(bot: Bot, chat_id: ChatId, message: &str) {
    let result = bot.send_message(chat_id, message).await;

    match result {
        Ok(_) => {}
        Err(err) => {
            sentry::capture_error(&err);
        }
    }
}

async fn on_receive(state: State, bot: Bot, msg: Message) {
    let user_request = find_user_by_username(&state, msg.chat.username().unwrap());

    let bot_cloned = bot.clone();
    let typing_interval = set_interval!(
        move || {
            tokio::spawn(send_typing_action(bot_cloned.clone(), msg.chat.id));
        },
        3000
    );

    if let Some(user) = user_request {
        proccess_message(user.clone(), bot, msg).await;
        clear_timer!(typing_interval)
    } else {
        send_message(bot, msg.chat.id, "Access denied").await;
        clear_timer!(typing_interval)
    }
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
            db.save_message(msg.chat.id, Role::Assistant, content.clone());
            send_message(bot, msg.chat.id, &content).await;
        }
        Err(error) => {
            send_message(bot, msg.chat.id, "I broke down. I feel bad").await;

            let error_ref: &dyn Error = &*error;
            sentry::capture_error(error_ref);
        }
    }
}

fn find_user_by_username<'a>(state: &'a State, username: &'a str) -> Option<&'a User> {
    state.users.iter().find(|user| user.user_name == username)
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
            tokio::spawn(on_receive(cloned_state, bot, msg));

            Ok(())
        };
        async move { fut.await }
    })
    .await;
}
