use std::{collections::HashSet, str::FromStr};

use gaia::{
    AppData,
    Error,
    commands::{
        arbi::{upcoming_arbitration, upcoming_arbitrations},
        archon_hunt::archon_hunt,
        event::events,
        market::market,
        worldstate::worldstate,
    },
    handle_error,
    init_db,
    notifier,
    utils::DbExtension,
};
use poise::{
    FrameworkContext,
    FrameworkError,
    serenity_prelude::{self, ClientBuilder, FullEvent, GatewayIntents, UserId},
};
use tokio::signal;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(LevelFilter::from_str(
            &std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_owned()),
        )?)
        .init();

    let token = std::env::var("BOT_TOKEN").expect("missing BOT_TOKEN env variable");
    let intents = GatewayIntents::non_privileged();

    let db = init_db().await?;

    let data = AppData::try_new_auto(db)?;
    let data_clone = data.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                worldstate(),
                upcoming_arbitration(),
                upcoming_arbitrations(),
                archon_hunt(),
                notifier::commands::notifier(),
                market(),
                events(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            on_error: |err: FrameworkError<'_, AppData, Error>| Box::pin(handle_error(err)),
            owners: HashSet::from_iter([UserId::new(350749990681051149)]),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                notifier::setup(ctx.clone(), data_clone.clone()).await?;

                Ok(data_clone.clone())
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    tokio::select! {
        res = client.start() => {
            if let Err(why) = res {
                tracing::error!("Client error: {:?}", why);
            }
        },
        _ = signal::ctrl_c() => {
            tracing::info!("Shutting down gracefully...");
            data.db().close().await;
        },
    }

    Ok(())
}

async fn event_handler(
    _ctx: &serenity_prelude::Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, AppData, Error>,
    data: &AppData,
) -> Result<(), Error> {
    // Remove all db entries for that guild upon the bot leaving/guild being deleted
    if let FullEvent::GuildDelete { incomplete, .. } = event {
        data.db()
            .delete_all_subscriptions(incomplete.id.get() as i64)
            .await?
    }

    Ok(())
}
