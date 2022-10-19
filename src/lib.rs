mod weather;

use anyhow::{anyhow, Context};
use serenity::model::gateway::Ready;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::prelude::*;
use serenity::{async_trait, model::prelude::GuildId};
use shuttle_secrets::SecretStore;
use tracing::info;

struct Bot {
    weather_api_key: String,
    client: reqwest::Client,
    discord_guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId(628079435832098827);

        let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands.create_application_command(|command| {
                command.name("hello").description("say hello")
            })
        })
        .await
        .unwrap();

        info!("{:?}", commands);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let response_content = match command.data.name.as_str() {
                "hello" => "hello".to_owned(),
                command => unreachable!("Unknown command: {}", command),
            };

            let create_interaction_response =
                command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(response_content))
                });

            if let Err(why) = create_interaction_response.await {
                eprintln!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[shuttle_service::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_service::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let weather_api_key = secret_store
        .get("WEATHER_API_KEY")
        .context("'WEATHER_API_KEY' was not found")?;

    let discord_guild_id = secret_store
        .get("DISCORD_GUILD_ID")
        .context("'DISCORD_GUILD_ID' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot {
            weather_api_key,
            client: reqwest::Client::new(),
            discord_guild_id: GuildId(discord_guild_id),
        })
        .await
        .expect("Err creating client");

    Ok(client)
}
