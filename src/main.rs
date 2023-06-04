use crate::command::on_receive_command;
use crate::db::DB;
use crate::utils::*;
use dotenv::dotenv;
use log::LevelFilter;

use std::sync::{Arc, Mutex};
use teloxide::{prelude::*, Bot};

mod command;
mod db;
mod gpt;
mod utils;

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
