#![feature(trait_alias)]

use log::error;
use serde::{Deserialize, Serialize};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{ActivityData, CreateEmbed};

mod persistent_data;
mod wake_on_lan;

#[derive(Deserialize, Serialize)]
struct State {
	foo: String,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, State, Error>;

#[poise::command(slash_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
	let author = ctx.author();
	let user_name = author.global_name.as_ref().unwrap_or(&author.name);

	ctx.send(CreateReply::default()
		.embed(CreateEmbed::new()
			.title("Hello, World!")
			.description(format!("Hello {user_name}, I'm a new bot!"))
			.colour(serenity::Colour::BLURPLE)
		)
		.ephemeral(true)
	).await?;

	Ok(())
}

#[poise::command(slash_command)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
	poise::builtins::register_application_commands_buttons(ctx).await?;
	Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let token = match std::env::var("DISCORD_TOKEN") {
		Ok(token) => token,
		Err(_) => {
			error!("Please configure the DISCORD_TOKEN environment variable");
			return;
		}
	};

	let intents = serenity::GatewayIntents::non_privileged();

	let framework = poise::Framework::builder()
		.options(poise::FrameworkOptions {
			commands: vec![hello(), register()],

			..Default::default()
		})
		.setup(|ctx, _ready, framework| {
			Box::pin(async move {
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(State { foo: "bar".into() })
			})
		})
		.build();

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(framework)
		.activity(ActivityData::playing("Taking over the world"))
		.await;
	client.unwrap().start().await.unwrap();
}
