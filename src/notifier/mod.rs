use std::sync::Arc;

use poise::serenity_prelude::{self, async_trait};

use crate::{notifier::s_tier_arbitrations::STierArbitrationListener, Data, Error};

pub mod commands;
pub mod model;
pub mod s_tier_arbitrations;

#[async_trait]
pub trait Notifier: Sized {
    async fn run(&self, ctx: serenity_prelude::Context, data: Arc<Data>) -> Result<(), Error>;
}

pub fn setup(ctx: serenity_prelude::Context, data: Arc<Data>) -> Result<(), Error> {
    tokio::spawn(STierArbitrationListener.run(ctx, data));

    Ok(())
}
