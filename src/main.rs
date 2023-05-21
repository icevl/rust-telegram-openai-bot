use crate::gpt::MyGPT;
use dotenv::dotenv;
use teloxide::prelude::*;

mod gpt;
mod redis;

lazy_static::lazy_static! {
    static ref GPT: MyGPT = {
        let api_key = std::env::var("GPT_KEY").expect("GPT_KEY must be set.");
        MyGPT::new(&api_key)
    };
}

async fn on_receive(bot: Bot, msg: Message) {
    let is_conversation_exists = GPT.conversation_exists(msg.chat.id);
    if !is_conversation_exists {
        GPT.new_chat_conversation(msg.chat.id);
        log::info!("New conversation created for chat id: {}", msg.chat.id);
    }

    let received = msg.text().unwrap();
    let result = GPT.send_msg(&received).await;

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
            eprintln!("Error: {}", err);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting...");

    let bot_token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set.");
    let bot = Bot::new(bot_token);

    teloxide::repl(bot, move |bot: Bot, msg: Message| async move {
        on_receive(bot, msg).await;
        Ok(())
    })
    .await;
}
