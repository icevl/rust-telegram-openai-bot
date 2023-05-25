use crate::utils::{find_user_by_username, send_message, State};
use std::str::FromStr;
use teloxide::{prelude::*, utils::command::BotCommands};

use crate::db::DB;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "New conversation")]
    New,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "help" => Ok(Command::Help),
            "new" => Ok(Command::New),
            _ => Err(()),
        }
    }
}

pub async fn on_receive_command(state: State, bot: Bot, msg: Message) {
    let user_request = find_user_by_username(&state, msg.chat.username().unwrap());
    let message = msg.text().unwrap();
    let (_, command) = message.split_at(1);
    let db = DB::new();

    if let Some(_user) = user_request {
        match Command::from_str(command) {
            Ok(cmd) => match cmd {
                Command::Help => {
                    send_message(bot, msg.chat.id, &Command::descriptions().to_string()).await;
                }

                Command::New => {
                    db.drop_history(msg.chat.id);
                    send_message(bot, msg.chat.id, "New conversation started").await;
                }
            },
            Err(_) => {}
        }
    }
}
