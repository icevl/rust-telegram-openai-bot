use dotenv::dotenv;
use teloxide::prelude::*;
use crate::gpt::MyGPT;

mod gpt;
mod redis;

lazy_static::lazy_static! {

    static ref GPT: MyGPT = {
        let api_key = std::env::var("GPT_KEY").expect("GPT_KEY must be set.");
        MyGPT::new(&api_key)
    };
}

async fn on_receive(bot: Bot, msg: Message) {
    let is_new_conversation = GPT.conversation_exists(msg.chat.id);
    if !is_new_conversation {
        GPT.new_chat_conversation(msg.chat.id);
        println!("New conversation created for chat id: {}", msg.chat.id);
    }

    // let redis_key = format!("chat_id_{}", msg.chat.id).to_string();

    // match redis::get_key(&redis_key) {
    //     Ok(value) => {
    //         println!("Redis key value {}: {}", redis_key, value);
    //     }
    //     Err(err) => {
    //         eprintln!("Redis key error {}: {}", redis_key, err);
    //     }
    // }

    // println!("redis: {}", conversation);

    let received = msg.text().unwrap();
    let result = GPT.send_msg(&received).await;

    match result {
        Ok(content) => {
            println!("Received content: {}", content);
            let send_result = bot.send_message(msg.chat.id, content).await;
            match send_result {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error: {}", err);
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
