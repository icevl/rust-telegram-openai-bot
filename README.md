# Quick start

## install Rust
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Database
Program using SQlite database

After the bot's first launch, a database file *database.db* will be created. 
Please kindly add your telegram username (remove first char '@') to the list of authorized names and indicate the appropriate method.

Schema:

 - users (authorized users)
 - chat_history (history messages for GPT conversation)


## Run in development mode with hot reload
```shell
cargo watch -x run
```

# Bot commands
- /help - *print help*
- /new - *start new conversation with new history*

# Build release

```shell
cargo build --release
```