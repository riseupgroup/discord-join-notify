use {
    ron::ser::PrettyConfig,
    serenity::{
        all::{Context, EventHandler, GatewayIntents, Ready, UserId, VoiceState},
        async_trait, Client,
    },
    std::{path::Path, sync::OnceLock},
    teloxide::{
        prelude::{Request, Requester},
        types::ChatId,
        Bot,
    },
    tracing_subscriber::FmtSubscriber,
};

mod config {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct User {
        pub name: String,
        pub discord_primary_id: u64,
        pub discord_secondary_ids: Vec<u64>,
        pub telegram_chat_id: Option<i64>,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct Config {
        pub discord_bot_token: String,
        pub telegram_bot_token: String,
        pub users: Vec<User>,
    }
}

use config::{Config, User as ConfigUser};

#[derive(Debug)]
struct User {
    name: String,
    discord_primary_id: UserId,
    discord_secondary_ids: Vec<UserId>,
    telegram_chat_id: Option<ChatId>,
}

impl User {
    fn has_discord_id(&self, id: UserId) -> bool {
        self.discord_primary_id == id || self.discord_secondary_ids.contains(&id)
    }
}

static TELEGRAM_BOT: OnceLock<Bot> = OnceLock::new();
static USERS: OnceLock<Vec<User>> = OnceLock::new();

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        log::info!("Bot is online as {}", ready.user.name);
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        if old_state.is_some_and(|state| state.guild_id == new_state.guild_id) {
            return;
        }
        if new_state.channel_id.is_none() {
            return;
        }

        let guild_id = new_state.guild_id.unwrap();
        let guild = guild_id.to_guild_cached(&ctx.cache).unwrap().clone();

        let telegram_bot = TELEGRAM_BOT.get().unwrap();
        let users = USERS.get().unwrap();

        let Some(user) = users.iter().find(|x| x.has_discord_id(new_state.user_id)) else {
            return;
        };
        if guild
            .voice_states
            .iter()
            .any(|(&id, _)| id != new_state.user_id && user.has_discord_id(id))
        {
            return;
        }

        if !guild
            .voice_states
            .iter()
            .any(|(&id, _)| id == user.discord_primary_id)
        {
            if let Some(chat_id) = user.telegram_chat_id {
                let discord_username = new_state.user_id.to_user(&ctx).await.unwrap().name;
                match telegram_bot
                    .send_message(
                        chat_id,
                        format!("You joined with {} on {}", discord_username, guild.name),
                    )
                    .send()
                    .await
                {
                    Ok(_) => log::info!("Sent telegram message to {}", user.name),
                    Err(err) => {
                        log::error!("Error sending telegram message to {}: {}", user.name, err)
                    }
                }
            }
        }

        let active_discord_accounts: Vec<_> = guild
            .voice_states
            .iter()
            .filter(|(_, state)| state.channel_id.is_some() && !state.self_deaf)
            .map(|(&id, _)| id)
            .collect();

        let message = format!("{} joined on {}!", user.name, guild.name);

        for other_user in users {
            if other_user.discord_primary_id == user.discord_primary_id {
                continue;
            }
            if !active_discord_accounts
                .iter()
                .any(|&id| other_user.has_discord_id(id))
            {
                if let Some(chat_id) = other_user.telegram_chat_id {
                    match telegram_bot.send_message(chat_id, &message).send().await {
                        Ok(_) => log::info!("Sent telegram message to {}", other_user.name),
                        Err(err) => log::error!(
                            "Error sending telegram message to {}: {}",
                            other_user.name,
                            err
                        ),
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        std::env::set_var("RUST_BACKTRACE", "1");
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::WARN)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Error setting subscriber");

    println!(
        r#"
-----------------------------------------------------------------------------
 🔊 Discord Join Notify - a riseupgroup project 🚀
 📦 Available at:
    💾 GitHub:      https://github.com/riseupgroup/discord-join-notify
    🐳 Docker Hub:  https://hub.docker.com/r/riseupgroup/discord-join-notify

 🔗 Check out more cool projects at:
    📌 https://github.com/riseupgroup
-----------------------------------------------------------------------------
"#
    );

    let discord_bot_token = {
        let config = match std::fs::read_to_string("config.ron") {
            Ok(config) => config,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                log::error!("config.ron not found, take a look at example.config.ron");
                let path = Path::new("example.config.ron");
                if !path.exists() {
                    let example = Config {
                        discord_bot_token: String::from("<discord token>"),
                        telegram_bot_token: String::from("<telegram token>"),
                        users: vec![
                            ConfigUser {
                                name: String::from("User1"),
                                discord_primary_id: 1234567890,
                                discord_secondary_ids: vec![2345678901, 3456789012],
                                telegram_chat_id: Some(123456),
                            },
                            ConfigUser {
                                name: String::from("User2"),
                                discord_primary_id: 567891234,
                                discord_secondary_ids: vec![],
                                telegram_chat_id: None,
                            },
                        ],
                    };
                    let mut pretty_config = PrettyConfig::default();
                    pretty_config.struct_names = true;
                    let example = ron::ser::to_string_pretty(&example, pretty_config).unwrap();
                    std::fs::write(path, example).unwrap();
                    log::info!("created example.config.ron");
                }

                println!();
                panic!("config.ron not found, take a look at example.config.ron");
            }
            Err(err) => {
                panic!("{err:?}");
            }
        };
        let config: Config = ron::from_str(&config).unwrap();
        let users: Vec<User> = config
            .users
            .into_iter()
            .map(|user| User {
                name: user.name,
                discord_primary_id: UserId::new(user.discord_primary_id),
                discord_secondary_ids: user
                    .discord_secondary_ids
                    .into_iter()
                    .map(UserId::new)
                    .collect(),
                telegram_chat_id: user.telegram_chat_id.map(ChatId),
            })
            .collect();
        USERS.set(users).unwrap();
        TELEGRAM_BOT
            .set(Bot::new(config.telegram_bot_token))
            .unwrap();
        config.discord_bot_token
    };

    let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILDS;
    let mut discord_client = Client::builder(discord_bot_token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let shard_manager = discord_client.shard_manager.clone();
    let mut handle = tokio::spawn(async move {
        discord_client.start().await.unwrap();
    });

    #[cfg(unix)]
    {
        use tokio::signal::unix;
        let mut interrupt = unix::signal(unix::SignalKind::interrupt()).unwrap();
        let mut terminate = unix::signal(unix::SignalKind::terminate()).unwrap();
        tokio::select! {
            _ = interrupt.recv() => (),
            _ = terminate.recv() => (),
            _ = &mut handle => return,
        }
    }
    #[cfg(not(unix))]
    tokio::select! {
        x = tokio::signal::ctrl_c() => x.unwrap(),
        _ = &mut handle => return
    }

    log::info!("Reveived stop signal, shutting down");
    shard_manager.shutdown_all().await;

    handle.await.unwrap();
}
