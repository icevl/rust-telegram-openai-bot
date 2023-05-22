use crate::db::DB;
use chatgpt::prelude::{ChatGPT, Conversation};
use futures::lock::Mutex;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use teloxide::prelude::ChatId;

pub struct MyGPT {
    client: ChatGPT,
    chat_conversations: Mutex<HashMap<ChatId, Arc<Mutex<Conversation>>>>,
}

impl MyGPT {
    pub fn new(api_key: &str) -> Self {
        MyGPT {
            client: ChatGPT::new(api_key).unwrap(),
            chat_conversations: Mutex::new(HashMap::new()),
        }
    }

    pub async fn new_chat_conversation(&self, chat_id: ChatId) {
        let db = DB::new();
        let conversation: Conversation = self.client.new_conversation();

        // let history = db.get_message(chat_id);
        // match  history {
        //     Ok(hist) => { conversation.se }
        // }

        //

        //self.client.send_history()

        let mutex_conversation = Arc::new(Mutex::new(conversation));

        self.chat_conversations
            .lock()
            .await
            .insert(chat_id, mutex_conversation);
    }

    pub async fn conversation_exists(&self, chat_id: ChatId) -> bool {
        self.chat_conversations.lock().await.contains_key(&chat_id)
    }

    pub async fn get_conversation(&self, chat_id: ChatId) -> Option<Arc<Mutex<Conversation>>> {
        let chat_conversations = self.chat_conversations.lock().await;
        if let Some(mutex_conversation) = chat_conversations.get(&chat_id) {
            Some(Arc::clone(mutex_conversation))
        } else {
            None
        }
    }

    pub async fn send_msg(
        &self,
        chat_id: ChatId,
        message: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        if let Some(mutex_conversation) = self.get_conversation(chat_id).await {
            let mut conversation_guard = mutex_conversation.lock().await;
            let response = conversation_guard.send_message(message).await;

            let content = match response.unwrap().message_choices.get(0) {
                Some(choice) => choice.message.clone().content,
                None => return Err("No message choices found".into()),
            };
            Ok(content)
        } else {
            Err("Conversation not found".into())
        }
    }
}
