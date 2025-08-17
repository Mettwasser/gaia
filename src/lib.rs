pub mod closest;
pub mod commands;
pub mod emojis;
pub mod notifier;
pub mod utils;

use std::{env, path::PathBuf, sync::Arc, time::Duration};

use arbitration_data::model::{dict::LanguageDict, regions::ExportRegions};
use derive_more::Debug;
use moka::future::Cache;
use poise::{
    CreateReply,
    serenity_prelude::{self, CreateEmbed, colours::roles::DARK_RED},
};
use sqlx::SqlitePool;
use warframe::{market, worldstate};

use crate::commands::market::average::Statistics;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type CmdRet = std::result::Result<(), Error>;
pub type Context<'a> = poise::ApplicationContext<'a, AppData, Error>;

pub const DEFAULT_COLOR: u32 = 0x228b22;

#[derive(Clone, Debug)]
pub struct AppData {
    worldstate: worldstate::Client,
    market: Arc<market::Client>,
    #[debug(skip)]
    arbi_data: Arc<arbitration_data::ArbitrationData>,
    db: SqlitePool,
    market_statistic_cache: Cache<String, Statistics>,
}

impl AppData {
    pub fn try_new_auto(pool: SqlitePool) -> Result<Self, Error> {
        let arbi_time_node_mapping =
            csv::Reader::from_reader(include_str!("../arbys.csv").as_bytes());
        let export_regions: ExportRegions<'_> =
            serde_json::from_str(include_str!("../regions.json"))?;
        let language_dict: LanguageDict = serde_json::from_str(include_str!("../dict.en.json"))?;

        let arbi_data = arbitration_data::ArbitrationData::new(
            arbi_time_node_mapping,
            export_regions,
            language_dict,
        )?;

        Ok(Self {
            #[cfg(debug_assertions)]
            worldstate: worldstate::Client::default(),
            #[cfg(not(debug_assertions))]
            // warframestatus is the container name here
            worldstate: worldstate::Client::new(reqwest::Client::new(), "http://warframestatus:3001".to_owned()),
            market: Arc::new(market::Client::new()),
            arbi_data: Arc::new(arbi_data),
            db: pool,
            market_statistic_cache: Cache::builder()
                .time_to_live(Duration::from_secs(60 * 60))
                .max_capacity(1000)
                .build(),
        })
    }

    pub fn worldstate(&self) -> &worldstate::Client {
        &self.worldstate
    }

    pub fn market(&self) -> &market::Client {
        &self.market
    }

    pub fn market_statistic_cache(&self) -> &Cache<String, Statistics> {
        &self.market_statistic_cache
    }

    pub fn arbi_data(&self) -> &arbitration_data::ArbitrationData {
        &self.arbi_data
    }

    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    pub fn db_owned(&self) -> SqlitePool {
        self.db.clone()
    }
}

type FrameworkError<'a> = poise::FrameworkError<'a, AppData, Error>;

pub async fn handle_error(err: FrameworkError<'_>) {
    use poise::FrameworkError::*;
    match err {
        Command { error, ctx, .. } => {
            tracing::warn!(error = %error, "Error in user command");
            handle_command_error(error, ctx).await.unwrap()
        },
        err => {
            tracing::error!(error = %err);
            poise::builtins::on_error(err).await.unwrap()
        },
    }
}

pub async fn handle_command_error(
    err: Error,
    ctx: poise::Context<'_, AppData, Error>,
) -> Result<(), serenity_prelude::Error> {
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::default()
                .title("Error")
                .description(err.to_string())
                .color(DARK_RED),
        ),
    )
    .await?;

    Ok(())
}

pub async fn init_db() -> Result<SqlitePool, Error> {
    let db_path = PathBuf::from(env::var("DATABASE_PATH").expect("DATABASE_PATH env var not set"));

    if !db_path.exists() {
        tracing::info!("Database file does not exist, creating a new one.");
        tokio::fs::File::create(db_path).await?;
    }

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not set");
    let pool = SqlitePool::connect(&db_url).await?;

    migrate(&pool).await?;

    Ok(pool)
}

async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
