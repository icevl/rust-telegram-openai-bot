use crate::db::DB;
use crate::gpt::MyGPT;
use dotenv::dotenv;
use log::LevelFilter;
use teloxide::{prelude::*, types::ChatAction};

mod db;
mod gpt;
mod redis;

lazy_static::lazy_static! {
    static ref GPT: MyGPT = {
        let api_key = std::env::var("GPT_KEY").expect("GPT_KEY must be set.");
        MyGPT::new(&api_key)
    };
}

async fn on_receive(bot: Bot, msg: Message) {
    let action = bot.send_chat_action(msg.chat.id, ChatAction::Typing).await;
    match action {
        Ok(_) => {}
        Err(_) => {}
    }

    let received = msg.text().unwrap();
    let result = GPT.send_msg(msg.chat.id, &received).await;

    log::info!("New message received {}", received);

    match result {
        Ok(content) => {
            log::info!("Received content: {}", content);
            let send_result = bot.send_message(msg.chat.id, content).await;

            match send_result {
                Ok(_) => {}
                Err(err) => {
                    log::info!("Error: {}", err);
                }
            }
        }
        Err(err) => {
            let error_msg = bot
                .send_message(msg.chat.id, "I broke down. I feel bad")
                .await;
            match error_msg {
                Ok(_) => {}
                Err(_) => {}
            }
            eprintln!("Error: {}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    let db = DB::new();
    db.init_migration().await;

    log::info!("Starting...");

    let bot_token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set.");
    let bot = Bot::new(bot_token);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        on_receive(bot, msg).await;
        Ok(())
    })
    .await;
}
