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
    #[command(description = "Broadcast message")]
    Broadcast,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "help" => Ok(Command::Help),
            "new" => Ok(Command::New),
            "text" => Ok(Command::Text),
            "voice" => Ok(Command::Voice),
            "broadcast" => Ok(Command::Broadcast),
            _ => Err(()),
        }
    }
}

pub async fn on_receive_command(
    state_users: Vec<User>,
    bot: Bot,
    msg: Message,
    state: Arc<Mutex<State>>,
) {
    let user_request = find_user_by_username(&state_users, msg.chat.username().unwrap());
    let message = msg.text().unwrap();
    let (_, command_line) = message.split_at(1);

    let substrings: Vec<&str> = command_line.split_whitespace().collect();
    let command = substrings[0];

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
                    db.disable_voice(&user.user_name);
                    let users_list = db.get_users().unwrap();
                    state.lock().unwrap().users = Mutex::new(users_list);

                    send_message(bot, msg.chat.id, "Text responses enabled").await;
                }

                Command::Voice => {
                    db.enable_voice(&user.user_name);
                    let users_list = db.get_users().unwrap();
                    state.lock().unwrap().users = Mutex::new(users_list);

                    send_message(bot, msg.chat.id, "Voice responses enabled").await;
                }

                Command::Broadcast => {
                    let text: String = substrings[1..].join(" ");
                    println!("text: {}", text);

                    let users_list = db.get_users().unwrap();
                    let mut users_count = 0;
                    for (_, user) in users_list.iter().enumerate() {
                        match user.chat_id {
                            Some(chat_id) => {
                                send_message(bot.clone(), chat_id, &text).await;
                                users_count += 1;
                            }
                            None => {}
                        }
                    }
                    let message = format!(
                        "Message successfully broadcaster for {} users!",
                        users_count
                    );

                    send_message(bot, msg.chat.id, &message).await;
                }
            },
            Err(_) => {}
        }
    }
}
