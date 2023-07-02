use crate::db::{User, DB};
use chatgpt::prelude::{ChatGPT, ChatGPTEngine, ModelConfigurationBuilder};
use chatgpt::types::{ChatMessage, Role};
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
        user: &User,
        message: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let db = DB::new();

        db.save_message(chat_id, Role::User, &message);

        let history = db.get_history(chat_id).unwrap();
        let enhanced_history = MyGPT::build_history(history, &user);

        print!("History: {:#?}", enhanced_history);

        let gpt_request = self.client.send_history(&enhanced_history).await;

        match gpt_request {
            Ok(response) => {
                let content = match response.message_choices.get(0) {
                    Some(choice) => choice.message.clone().content,
                    None => return Err("No message choices found".into()),
                };
                Ok(content)
            }
            Err(err) => Err(err.into()),
        }
    }

    fn build_history(history: Vec<ChatMessage>, user: &User) -> Vec<ChatMessage> {
        let mut updated_history = Vec::new();
        let user_name = user.contact_name.clone();
        let user_form = user.contact_form.clone();

        updated_history.push(ChatMessage {
            content: format!(
                "Называй меня {}. Говори со мной на {}, как будто мы с тобой давно знакомы",
                user_name, user_form
            )
            .to_string(),
            role: Role::User,
        });

        updated_history.push(ChatMessage {
            content: format!("Привет, {}! Конечно, мы можем общаться на '{}'. Как дела? Чем я могу тебе помочь? Меня зовут Валя",user_name, user_form),
            role: Role::Assistant,
        });

        updated_history.extend(history);
        updated_history
    }
}
