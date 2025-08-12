use std::sync::Arc;

use futures::future::join_all;
use poise::serenity_prelude::{
    self,
    CreateEmbed,
    CreateMessage,
    FormattedTimestamp,
    FormattedTimestampStyle,
    Mentionable,
    Timestamp,
};
use warframe::worldstate::{queryable::Cetus, CetusState, TimedEvent};

use crate::{
    notifier::{model::SubscriptionType, Notifier},
    utils::{self, ApplyIf, DbExtension},
    AppData,
    Error,
};

fn build_embed(cetus: &Cetus) -> CreateEmbed {
    utils::embed()
        .title("Eidolon Time!")
        .description(
            "Time for Eidolons!\nIt just turned night on Cetus, get ready for some hunting!",
        )
        .field(
            "Back to day in",
            FormattedTimestamp::new(
                cetus.expiry().into(),
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string(),
            false,
        )
        .timestamp(Timestamp::now())
}

pub struct EidolonHunts;

impl Notifier for EidolonHunts {
    async fn run(ctx: serenity_prelude::Context, data: Arc<AppData>) -> Result<(), Error> {
        data.worldstate
            .call_on_update_with_state::<_, Cetus, _>(callback, (ctx, data.clone()))
            .await
            .map_err(Error::from)
    }
}

async fn callback(
    (ctx, data): (serenity_prelude::Context, Arc<AppData>),
    _: &Cetus,
    cetus: &Cetus,
) {
    if cetus.state == CetusState::Day {
        return;
    }

    let subscriptions = data
        .db()
        .get_subscriptions(SubscriptionType::EidolonHunts)
        .await
        .unwrap_or_default();

    let embed = build_embed(cetus);

    let notification_tasks = subscriptions
        .iter()
        .map(|sub| async {
            let result = sub
                .notification_channel_id
                .send_message(
                    &ctx,
                    CreateMessage::new()
                        .apply_optionally(sub.role_id_to_mention, |msg, role_id| {
                            msg.content(role_id.mention().to_string())
                        })
                        .add_embed(embed.clone()),
                )
                .await;

            if let Err(e) = &result {
                tracing::error!(
                    channel_id = %sub.notification_channel_id,
                    error = %e,
                    "Failed to send notification for Eidolon Hunt",
                );
            }

            result
        })
        .collect::<Vec<_>>();

    join_all(notification_tasks).await;
}
