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
    pub is_voice: bool,
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
                contact_form    VARCHAR(20) NOT NULL,
                is_voice        TINNYINT(1) DEFAULT 0
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

    pub fn drop_history(&self, chat_id: ChatId) {
        let mut request = self
            .connection
            .prepare("DELETE FROM chat_history WHERE chat_id = :chat_id")
            .unwrap();
        request
            .execute(&[(":chat_id", &chat_id.to_string())])
            .unwrap();
    }

    pub fn get_history(&self, chat_id: ChatId) -> Result<Vec<ChatMessage>, rusqlite::Error> {
        let mut stmt = self.connection.prepare(
            "SELECT message, role FROM (SELECT message, role, created_at FROM chat_history WHERE chat_id = ? ORDER BY created_at DESC LIMIT 10) ORDER BY created_at ASC",
        )?;

        let message_iter = stmt
            .query_map([chat_id.to_string()], |row| {
                Ok(LoadedMessage {
                    content: row.get(0)?,
                    role: DB::string_to_role(row.get::<_, String>(1)?.as_str()),
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

    pub fn enable_voice(&self, user_name: String) {
        let mut request = self
            .connection
            .prepare("UPDATE users SET is_voice = 1 WHERE username = :user_name")
            .unwrap();
        request.execute(&[(":user_name", &user_name)]).unwrap();
    }

    pub fn disable_voice(&self, user_name: String) {
        let mut request = self
            .connection
            .prepare("UPDATE users SET is_voice = 0 WHERE username = :user_name")
            .unwrap();
        request.execute(&[(":user_name", &user_name)]).unwrap();
    }

    pub fn get_users(&self) -> Result<Vec<User>, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT username, contact_name, contact_form, is_voice FROM users")?;

        let users_iter = stmt
            .query_map([], |row| {
                Ok(User {
                    user_name: row.get(0)?,
                    contact_name: row.get(1)?,
                    contact_form: row.get(2)?,
                    is_voice: row.get(3)?,
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
                        is_voice: row.is_voice,
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
