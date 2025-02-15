# Discord to Telegram Notification Bot

This bot sends a Telegram notification whenever a person joins a voice channel on a Discord server. The message is sent to all users configured in the file `config.ron`, except for the person who joined. This bot takes the need of writing a message to all your friends when you want to meet.

## 🛠️ Configuration
- Run the bot with `cargo run`
    - The bot will create a `example.config.ron` if the file `config.ron` does not exist
- Fill in the api keys for discord and telegram
    - Obtain the discord token from the [Discord Developer Portal](https://discord.com/developers/applications)
    - Obtain the telegram token from the [BotFather](https://core.telegram.org/bots#6-botfather)
- `telegram_chat_id` is the chat (id) between a user and the bot
- `discord_primary_id` is the id of a users primary discord account
- `discord_secondary_id` is an array of a users secondary account ids

## 🐳 Docker

🔗 [riseupgroup/discord-join-notify](https://hub.docker.com/r/riseupgroup/discord-join-notify)

## 📢 Contributing

Feel free to **open issues** or **submit pull requests** on GitHub! 🚀

## 📜 License

This project is licensed under the MIT License.
