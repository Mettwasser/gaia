pub mod commands;
pub mod eidolon_hunts;
pub mod error;
pub mod model;
pub mod s_tier_arbitrations;
pub mod sp_disruption_fissure;

use std::{fmt::Debug, future::Future};

use poise::serenity_prelude::{self};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

use crate::{
    AppData,
    Error,
    notifier::{
        eidolon_hunts::EidolonHunts,
        error::{NotifierError, handle_notifier_error},
        s_tier_arbitrations::STierArbitrationListener,
        sp_disruption_fissure::SteelPathDisruptionFissures,
    },
};

#[derive(Debug, Clone)]
pub struct ListenerCallbackData {
    ctx: serenity_prelude::Context,
    data: AppData,
    tx: UnboundedSender<NotifierError>,
}

pub trait Notifier {
    fn run(
        ctx: serenity_prelude::Context,
        data: AppData,
        tx: UnboundedSender<NotifierError>,
    ) -> impl Future<Output = Result<(), Error>> + Send + 'static;
}

pub async fn setup(ctx: serenity_prelude::Context, data: AppData) -> Result<(), Error> {
    spawn_notifier::<STierArbitrationListener>(&ctx, &data)?;

    spawn_notifier::<SteelPathDisruptionFissures>(&ctx, &data)?;

    spawn_notifier::<EidolonHunts>(&ctx, &data)?;

    Ok(())
}

fn spawn_notifier<T>(ctx: &serenity_prelude::Context, data: &AppData) -> Result<(), Error>
where
    T: Notifier + Send + 'static,
{
    let ctx = ctx.clone();
    let data = data.clone();

    let (tx, rx) = unbounded_channel::<NotifierError>();

    tokio::spawn(handle_notifier_error(rx, data.clone()));
    tokio::spawn(async move {
        if let Err(e) = T::run(ctx, data, tx).await {
            tracing::error!(error = %e, "Notifier encountered an error");
        }
    });

    Ok(())
}
