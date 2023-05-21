use chatgpt::prelude::{ChatGPT, Conversation};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use teloxide::prelude::ChatId;

pub struct MyGPT {
    client: ChatGPT,
    chat_conversations: Mutex<HashMap<ChatId, Mutex<Conversation>>>,
}

impl MyGPT {
    pub fn new(api_key: &str) -> Self {
        MyGPT {
            client: ChatGPT::new(api_key).unwrap(),
            chat_conversations: Mutex::new(HashMap::new()),
        }
    }

    pub fn new_chat_conversation(&self, chat_id: ChatId) {
        let conversation: Conversation = self.client.new_conversation();
        let mutex_conversation = Mutex::new(conversation);
        self.chat_conversations
            .lock()
            .unwrap()
            .insert(chat_id, mutex_conversation);
    }

    pub fn conversation_exists(&self, chat_id: ChatId) -> bool {
        self.chat_conversations
            .lock()
            .unwrap()
            .contains_key(&chat_id)
    }

    // pub fn get_chat_conversation(&self, chat_id: &ChatId) -> Option<MutexGuard<Conversation>> {
    //     if let Some(conversation_mutex) = self.chat_conversations.lock().unwrap().get(chat_id) {
    //         Some(conversation_mutex.lock().unwrap())
    //     } else {
    //         None
    //     }
    // }

    pub fn new_conversation(&self) -> Conversation {
        self.client.new_conversation()
    }

    pub async fn send_msg(
        &self,
        //chat_id: ChatId,
        message: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response = self.client.send_message(message).await?;

        // let conversation = self.get_chat_conversation(&chat_id);
        // let response = match conversation {
        //     Some(mut conversation) => conversation.send_message(message).await?,
        //     None => return Err("Conversation not found".into()),
        // };

        let content = match response.message_choices.get(0) {
            Some(choice) => choice.message.clone().content,
            None => return Err("No message choices found".into()),
        };

        Ok(content)
    }
}
