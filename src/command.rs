use crate::db::{User, DB};
use crate::utils::{find_user_by_username, send_message, State};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use teloxide::{prelude::*, utils::command::BotCommands};

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
    #[command(description = "Text responses")]
    Text,
    #[command(description = "Voice responses")]
    Voice,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "help" => Ok(Command::Help),
            "new" => Ok(Command::New),
            "text" => Ok(Command::Text),
            "voice" => Ok(Command::Voice),
            _ => Err(()),
        }
    }
}

pub async fn on_receive_command(
    state_users: Vec<User>,
    bot: Bot,
    msg: Message,
    state: Arc<Mutex<State>>,
    //cb: impl FnOnce(&std::sync::Mutex<Vec<User>>),
) {
    let user_request = find_user_by_username(&state_users, msg.chat.username().unwrap());
    let message = msg.text().unwrap();
    let (_, command) = message.split_at(1);
    let db = DB::new();

    if let Some(user) = user_request {
        match Command::from_str(command) {
            Ok(cmd) => match cmd {
                Command::Help => {
                    send_message(bot, msg.chat.id, &Command::descriptions().to_string()).await;
                }

                Command::New => {
                    db.drop_history(msg.chat.id);
                    send_message(bot, msg.chat.id, "New conversation started").await;
                }

                Command::Text => {
                    db.disable_voice(user.user_name.to_string());
                    let users_list = db.get_users().unwrap();
                    state.lock().unwrap().users = Mutex::new(users_list);

                    send_message(bot, msg.chat.id, "Text responses enabled").await;
                }

                Command::Voice => {
                    db.enable_voice(user.user_name.to_string());
                    let users_list = db.get_users().unwrap();
                    state.lock().unwrap().users = Mutex::new(users_list);

                    send_message(bot, msg.chat.id, "Voice responses enabled").await;
                }
            },
            Err(_) => {}
        }
    }
}
