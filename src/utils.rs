use crate::db::User;
use teloxide::{prelude::*, types::ChatAction};

#[derive(Clone, Debug)]
pub struct State {
    pub users: Vec<User>,
}

pub fn find_user_by_username<'a>(state: &'a State, username: &'a str) -> Option<&'a User> {
    state.users.iter().find(|user| user.user_name == username)
}

pub async fn send_message(bot: Bot, chat_id: ChatId, message: &str) {
    let result = bot.send_message(chat_id, message).await;

    match result {
        Ok(_) => {}
        Err(err) => {
            sentry::capture_error(&err);
        }
    }
}

pub async fn send_typing_action(bot: Bot, chat_id: ChatId) {
    match bot.send_chat_action(chat_id, ChatAction::Typing).await {
        Ok(_) => {}
        Err(err) => {
            sentry::capture_error(&err);
        }
    };
}

pub fn is_command_message(msg: Message) -> bool {
    let message = msg.text().unwrap();
    let first_char = message.chars().nth(0).unwrap();
    if first_char == '/' {
        return true;
    }
    return false;
}
