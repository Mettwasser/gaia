pub mod commands;
pub mod eidolon_hunts;
pub mod model;
pub mod s_tier_arbitrations;
pub mod sp_disruption_fissure;

use std::future::Future;

use poise::serenity_prelude::{self};

use crate::{
    AppData,
    Error,
    notifier::{
        eidolon_hunts::EidolonHunts,
        s_tier_arbitrations::STierArbitrationListener,
        sp_disruption_fissure::SteelPathDisruptionFissures,
    },
};

pub trait Notifier {
    fn run(
        ctx: serenity_prelude::Context,
        data: AppData,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static;
}

pub async fn setup(ctx: serenity_prelude::Context, data: AppData) -> Result<(), Error> {
    // we need to artificially delay task creation to not be blocked by cloudflare
    // (for warframestat.us)
    spawn_notifier::<STierArbitrationListener>(&ctx, &data)?;

    spawn_notifier::<SteelPathDisruptionFissures>(&ctx, &data)?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    spawn_notifier::<EidolonHunts>(&ctx, &data)?;

    Ok(())
}

fn spawn_notifier<T>(ctx: &serenity_prelude::Context, data: &AppData) -> Result<(), Error>
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
