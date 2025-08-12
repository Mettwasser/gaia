use std::{future::Future, sync::Arc};

use poise::serenity_prelude::{self};

use crate::{
    notifier::{
        eidolon_hunts::EidolonHunts,
        s_tier_arbitrations::STierArbitrationListener,
        sp_disruption_fissure::SteelPathDisruptionFissuresListener,
    },
    AppData,
    Error,
};

pub mod commands;
pub mod eidolon_hunts;
pub mod model;
pub mod s_tier_arbitrations;
pub mod sp_disruption_fissure;

pub trait Notifier {
    fn run(
        ctx: serenity_prelude::Context,
        data: Arc<AppData>,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static;
}

pub fn setup(ctx: serenity_prelude::Context, data: Arc<AppData>) -> Result<(), Error> {
    spawn_notifier::<STierArbitrationListener>(&ctx, &data)?;
    spawn_notifier::<SteelPathDisruptionFissuresListener>(&ctx, &data)?;
    spawn_notifier::<EidolonHunts>(&ctx, &data)?;

    Ok(())
}

fn spawn_notifier<T>(ctx: &serenity_prelude::Context, data: &Arc<AppData>) -> Result<(), Error>
where
    T: Notifier + Send + 'static,
{
    let ctx = ctx.clone();
    let data = data.clone();

    tokio::spawn(async move {
        if let Err(e) = T::run(ctx, data).await {
            tracing::error!(error = %e, "Notifier encountered an error");
        }
    });

    Ok(())
}
