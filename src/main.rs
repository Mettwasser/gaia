use std::{collections::HashSet, sync::Arc};

use gaia::{
    commands::{
        arbi::{upcoming_arbitration, upcoming_arbitrations},
        archon_hunt::archon_hunt,
        worldstate::worldstate,
    },
    handle_error,
    init_db,
    notifier,
    Data,
    Error,
};
use poise::{
    serenity_prelude::{ClientBuilder, GatewayIntents, UserId},
    FrameworkError,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().unwrap();
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .init();

    let token = std::env::var("BOT_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = GatewayIntents::privileged().difference(GatewayIntents::MESSAGE_CONTENT);

    let db = init_db().await?;

    let data = Arc::new(Data::try_new_auto(db)?);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                worldstate(),
                upcoming_arbitration(),
                upcoming_arbitrations(),
                archon_hunt(),
                notifier::commands::notifier(),
            ],
            on_error: |err: FrameworkError<'_, Arc<Data>, Error>| Box::pin(handle_error(err)),
            owners: HashSet::from_iter([UserId::new(350749990681051149)]),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                notifier::setup(ctx.clone(), data.clone())?;
                Ok(data)
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    client.start().await.unwrap();

    Ok(())
}
