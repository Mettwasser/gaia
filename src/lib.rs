pub mod closest;
pub mod commands;
pub mod embed_paginator;
mod macros;
pub mod paginate;
pub mod utils;

use std::sync::Arc;

use arbitration_data::model::{dict::LanguageDict, regions::ExportRegions};
use poise::serenity_prelude::{self, Color, CreateEmbed};
use utils::embed;
use warframe::worldstate::prelude as wf;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type CmdRet = std::result::Result<(), Error>;
pub type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

pub const DEFAULT_COLOR: u32 = 0x228b22;

pub struct Data {
    client: wf::Client,
    arbi_data: arbitration_data::ArbitrationData,
}

impl Data {
    pub fn try_new_auto() -> Result<Self, Error> {
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
            client: wf::Client::new(),
            arbi_data,
        })
    }

    pub fn client(&self) -> &wf::Client {
        &self.client
    }

    pub fn arbi_data(&self) -> &arbitration_data::ArbitrationData {
        &self.arbi_data
    }
}

type FrameworkError<'a> = poise::FrameworkError<'a, Arc<Data>, Error>;

fn error_embed(description: impl Into<String>) -> CreateEmbed {
    embed()
        .title("Error")
        .description(description)
        .color(Color::RED)
}

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
    ctx: Context<'_>,
) -> Result<(), serenity_prelude::Error> {
    poise::builtins::on_error(poise::FrameworkError::new_command(ctx, err))
        .await
        .unwrap();

    Ok(())
}
