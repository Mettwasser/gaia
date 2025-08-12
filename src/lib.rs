pub mod closest;
pub mod commands;
pub mod notifier;
pub mod utils;

use std::{env, sync::Arc};

use arbitration_data::model::{dict::LanguageDict, regions::ExportRegions};
use poise::serenity_prelude;
use sqlx::SqlitePool;
use warframe::worldstate;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type CmdRet = std::result::Result<(), Error>;
pub type Context<'a> = poise::ApplicationContext<'a, Arc<AppData>, Error>;

pub const DEFAULT_COLOR: u32 = 0x228b22;

pub struct AppData {
    worldstate: worldstate::Client,
    arbi_data: arbitration_data::ArbitrationData,
    db: SqlitePool,
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
            worldstate: worldstate::Client::new(),
            arbi_data,
            db: pool,
        })
    }

    pub fn worldstate_client(&self) -> &worldstate::Client {
        &self.worldstate
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

type FrameworkError<'a> = poise::FrameworkError<'a, Arc<AppData>, Error>;

pub async fn handle_error(err: FrameworkError<'_>) {
    tracing::error!(error = %err);

    use poise::FrameworkError::*;
    match err {
        Command { error, ctx, .. } => handle_command_error(error, ctx).await.unwrap(),
        err => poise::builtins::on_error(err).await.unwrap(),
    }
}

pub async fn handle_command_error(
    err: Error,
    ctx: poise::Context<'_, Arc<AppData>, Error>,
) -> Result<(), serenity_prelude::Error> {
    poise::builtins::on_error(poise::FrameworkError::new_command(ctx, err))
        .await
        .unwrap();

    Ok(())
}

pub async fn init_db() -> Result<SqlitePool, Error> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env var not set");
    let pool = SqlitePool::connect(&db_url).await?;

    migrate(&pool).await?;

    Ok(pool)
}

async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
