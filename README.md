# Rarchbot

A Telegram bot that monitors the rss feed at https://archlinux.org/feeds/news/ and sends you updates.

Built with Rust, Tokio, Teloxide, and SQLx.

## Supported commands

- /help — display this text.
- /start — Will subscribe you to updates
- /stop — Will unsubscribe you from updates
- /last — Will send the latest post awailable
- /issubscribed — Will check your subsribtion status

## Features
- Periodically polls an RSS feed for new posts
- Sends new posts to all subscribed Telegram chats
- Persists posts and subscribers in a database
- Fully asynchronous using Tokio
- Graceful shutdown on Ctrl+C and SIGTERM
- Docker support
- Automatic database migrations
- SQLite support
- PostgreSQL support coming up

## Configuration

The bot is configured using environment variables.

| Variable | Description |
| --- | --- |
| `TELOXIDE_TOKEN` | Telegram Bot API token |
| `URL` | RSS/ feed URL to monitor |
| `DATABASE_URL` | SQLx database connection URL |

URL is optional if it's not set the [default](https://archlinux.org/feeds/news/) will be used. 

It can theoretically work with other feeds but results may be inconsistent.

## Installation

### Running a Release

Download the latest release from the Releases page.

Make the binary executable if necessary:

```
chmod +x rarchbot
```

Set the required environment variables:

```
export TELOXIDE_TOKEN="your_telegram_bot_token"
export URL="https://example.com/feed.xml"
export DATABASE_URL="sqlite:///data/rarchbot.db"
```

Then run the bot:

```
./rarchbot
```

### Running with Docker Compose

Set up .env file according to .env.example

Start the bot:
```
docker compose up -d
```
View logs:
```
docker compose logs -f
```
Stop the bot:
```
docker compose stop
```

### Building from source
If you want to build Rarchbot yourself, you'll need:

- Rust toolchain
- Cargo
- SQLite

Clone the repository:
```
git clone <repository-url>
cd rarchbot
```
Create your environment file:
```
cp .env.example .env
```
Configure the required environment variables.

Alternatively just set the required environment variables:

```
export TELOXIDE_TOKEN="your_telegram_bot_token"
export URL="https://example.com/feed.xml"
export DATABASE_URL="sqlite:///data/rarchbot.db"
```

Build the application:
```
cargo build --release
```
Run the bot:
```
./target/release/rarchbot
```
