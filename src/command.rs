use crate::libs::{find_user_by_username, send_message, State};
use teloxide::prelude::*;

use crate::db::DB;

pub async fn on_receive_command(state: State, bot: Bot, msg: Message) {
    let user_request = find_user_by_username(&state, msg.chat.username().unwrap());
    let message = msg.text().unwrap();
    let (_, command) = message.split_at(1);
    let db = DB::new();

    if let Some(_user) = user_request {
        match command {
            "clean" => {
                db.drop_history(msg.chat.id);
                send_message(bot, msg.chat.id, "Session cleared successfully").await
            }
            _ => {}
        }
    }
}
