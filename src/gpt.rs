use crate::db::DB;
use chatgpt::prelude::{ChatGPT, ChatGPTEngine, ModelConfigurationBuilder};
use chatgpt::types::Role;
use std::error::Error;
use teloxide::prelude::ChatId;

pub struct MyGPT {
    client: ChatGPT,
}

impl MyGPT {
    pub fn new(api_key: &str) -> Self {
        MyGPT {
            client: ChatGPT::new_with_config(
                api_key,
                ModelConfigurationBuilder::default()
                    .temperature(1.0)
                    .engine(ChatGPTEngine::Gpt35Turbo)
                    .build()
                    .unwrap(),
            )
            .unwrap(),
        }
    }

    pub async fn send_msg(
        &self,
        chat_id: ChatId,
        message: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let db = DB::new();

        db.save_message(chat_id, Role::User, message.to_string());

        let history = db.get_message(chat_id).unwrap();
        print!("H: {:#?}", history);

        let gpt_request = self.client.send_history(&history).await;

        match gpt_request {
            Ok(response) => {
                let content = match response.message_choices.get(0) {
                    Some(choice) => choice.message.clone().content,
                    None => return Err("No message choices found".into()),
                };
                Ok(content)
            }
            Err(err) => {
                print!("Error: {:#?}", err);
                Err("An error occurred ".into())
            }
        }
    }
}
