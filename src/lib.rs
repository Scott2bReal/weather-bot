mod weather;

use anyhow::Context as _;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::{Interaction, InteractionResponseType};
use serenity::prelude::*;
use serenity::{async_trait, model::prelude::GuildId};
use shuttle_secrets::SecretStore;
use tracing::info;

const OPTIONS_PLACE: &str = "place";
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
            commands
                // Basic hello command
                .create_application_command(|command| {
                    command.name("hello").description("say hello")
                })
                // Command to look up weather
                .create_application_command(|command| {
                    command
                        .name("weather")
                        .description("Display the weather")
                        .create_option(|option| {
                            option
                                .name("place")
                                .description("City to look up")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
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
                "weather" => {
                    let argument = command
                        .data
                        .options
                        .iter()
                        .find(|opt| opt.name == OPTIONS_PLACE)
                        .cloned();

                    let value = argument.unwrap().value.unwrap();
                    let place = value.as_str().unwrap();
                    let result = weather::get_forecast(place, &self.weather_api_key, &self.client).await;
                    info!("{:?}", result);

                    match result {
                        Ok((location, forecast)) => {
                            info!("{:?}", &forecast);
                            format!("Forecast: {} in {}", forecast.headline.overview, location)
                        }
                        Err(err) => {
                            format!("Err: {}", err)
                        }
                    }
                }
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
            discord_guild_id: GuildId(discord_guild_id.parse().unwrap()),
        })
        .await
        .expect("Err creating client");

    Ok(client)
}
