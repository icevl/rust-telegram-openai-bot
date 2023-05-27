# Overview
This is a simple Rust project that implements a Telegram bot integrated with OpenAI. In addition, this project enables customization of OpenAI responses with a Text-to-Speech (TTS) API capable of generating voice messages.

For the TTS functionality, you can use a self-hosted TTS engine, which is demonstrated in my project [python-silero-http-api](https://github.com/icevl/python-silero-http-api). This project using [silero-models](https://github.com/snakers4/silero-models), although other TTS services can be used as well.

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

## Env
Setup .env file based on .env.example
```
GPT_KEY=<OpenAI token>
TELEGRAM_TOKEN=<Bot token>
SENTRY_DSN=<optional sentry dsn>
TTS_PATH=<optional tts path> (example: http://localhost:10000/)
```

## Run in development mode with hot reload
```shell
cargo watch -x run
```

# Bot commands
- /help - *print help*
- /new - *start new conversation with new history*
- /text - *text responses*
- /voice - *voice responses*

# Build release

```shell
cargo build --release
```
