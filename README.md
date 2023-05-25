# Quick start

## install Rust
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Database
Program using SQLite database.

Once the bot is launched for the first time, a database file named 'database.db' will be created. Please add your Telegram username to the list of authorized names in *users* table (without the first '@' symbol)

**Schema:**

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
