use chatgpt::types::{ChatMessage, Role};
use rusqlite::{Connection, Result};
use teloxide::prelude::ChatId;

pub struct DB {
    connection: Connection,
}

struct Message {
    chat_id: String,
    message: String,
    role: String,
}

struct LoadedMessage {
    content: String,
    role: Role,
}

#[derive(Clone, Debug)]
pub struct User {
    pub user_name: String,
    pub contact_name: String,
    pub contact_form: String,
}

impl DB {
    pub fn new() -> Self {
        DB {
            connection: Connection::open("database.db").unwrap(),
        }
    }

    pub async fn history_migration(&self) {
        let result = self.connection.execute(
            "CREATE TABLE chat_history (
                id          INTEGER PRIMARY KEY,
                chat_id     INTEGER  NOT NULL,
                message     TEXT NOT NULL,
                role        VARCHAR(20) NOT NULL,
                created_at  TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            (),
        );

        match result {
            Ok(_) => {
                log::info!("Table [history] successfully created")
            }
            Err(err) => {
                log::info!("Error in [history] creation: {}", err)
            }
        }
    }

    pub async fn users_migration(&self) {
        let result = self.connection.execute(
            "CREATE TABLE users (
                id              INTEGER PRIMARY KEY,
                username        VARCHAR(100) NOT NULL,
                contact_name    VARCHAR(100) NOT NULL,
                contact_form    VARCHAR(20) NOT NULL
            )",
            (),
        );

        match result {
            Ok(_) => {
                log::info!("Table [users] successfully created")
            }
            Err(err) => {
                log::info!("Error in [users] creation: {}", err)
            }
        }
    }

    pub fn save_message(&self, chat_id: ChatId, role: Role, message: String) {
        let msg_data = Message {
            chat_id: chat_id.to_string(),
            message: message,
            role: DB::role_to_string(role),
        };

        self.connection
            .execute(
                "INSERT INTO chat_history (chat_id, message, role) VALUES (?1, ?2, ?3)",
                (&msg_data.chat_id, &msg_data.message, &msg_data.role),
            )
            .unwrap();
    }

    pub fn get_message(&self, chat_id: ChatId) -> Result<Vec<ChatMessage>, rusqlite::Error> {
        let mut stmt = self.connection.prepare(
            "SELECT message FROM chat_history WHERE chat_id = ? ORDER BY created_at ASC LIMIT 100",
        )?;

        let message_iter = stmt
            .query_map([chat_id.to_string()], |row| {
                Ok(LoadedMessage {
                    content: row.get(0)?,
                    role: Role::User,
                })
            })
            .unwrap();

        let chat_messages: Result<Vec<ChatMessage>, rusqlite::Error> =
            message_iter.collect::<Result<Vec<_>, _>>().map(|messages| {
                messages
                    .into_iter()
                    .map(|loaded_message| ChatMessage {
                        content: loaded_message.content,
                        role: loaded_message.role,
                    })
                    .collect()
            });

        return chat_messages;
    }

    pub fn get_users(&self) -> Result<Vec<User>, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT username, contact_name, contact_form FROM users")?;

        let users_iter = stmt
            .query_map([], |row| {
                Ok(User {
                    user_name: row.get(0)?,
                    contact_name: row.get(1)?,
                    contact_form: row.get(2)?,
                })
            })
            .unwrap();

        let users: Result<Vec<User>, rusqlite::Error> =
            users_iter.collect::<Result<Vec<_>, _>>().map(|users| {
                users
                    .into_iter()
                    .map(|row| User {
                        user_name: row.user_name,
                        contact_name: row.contact_name,
                        contact_form: row.contact_form,
                    })
                    .collect()
            });

        return users;
    }

    fn role_to_string(role: Role) -> String {
        match role {
            Role::System => "system".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::User => "user".to_string(),
        }
    }

    fn string_to_role(role_str: &str) -> Role {
        match role_str {
            "system" => Role::System,
            "assistant" => Role::Assistant,
            "user" => Role::User,
            _ => panic!("Invalid role"),
        }
    }
}
