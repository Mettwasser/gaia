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
use warframe::worldstate::{Change, MissionType, Tier, TimedEvent, queryable::Fissure};

use crate::{
    AppData,
    Error,
    notifier::{Notifier, model::SubscriptionType},
    utils::{self, ApplyIf, DbExtension},
};

fn build_embed(fissure: &Fissure) -> CreateEmbed {
    utils::embed()
        .title("New Steel Path Disruption Fissure")
        .field("Node", &fissure.node, true)
        .field("Mission Type", &fissure.mission_type, true)
        .field("Tier", fissure.tier.to_string(), true)
        .field(
            "Ends",
            FormattedTimestamp::new(
                fissure.expiry().into(),
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string(),
            false,
        )
        .timestamp(Timestamp::now())
}

pub struct SteelPathDisruptionFissures;

impl Notifier for SteelPathDisruptionFissures {
    async fn run(ctx: serenity_prelude::Context, data: AppData) -> Result<(), Error> {
        data.worldstate()
            .call_on_nested_update_with_state::<_, Fissure, _>(callback, (ctx, data.clone()))
            .await
            .map_err(Error::from)
    }
}

async fn callback(
    (ctx, data): (serenity_prelude::Context, AppData),
    fissure: &Fissure,
    change: Change,
) {
    if !is_correct_fissure(fissure, change) {
        return;
    }

    let subscriptions = data
        .db()
        .get_subscriptions(SubscriptionType::SteelPathDisruptionFissures)
        .await
        .unwrap_or_default();

    let embed = build_embed(fissure);

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
                    "Failed to send notification for Steel Path Disruption Fissure",
                );
            }

            result
        })
        .collect::<Vec<_>>();

    join_all(notification_tasks).await;
}

/// Checks if a Steel Path Disruption Fissure is valid for notification.
///
/// It checks:
/// - Tier is NOT requiem
/// - Mission Type is Disruption
/// - Fissure is Hard (Steel Path)
fn is_correct_fissure(fissure: &Fissure, change: Change) -> bool {
    change == Change::Added
        && fissure.tier != Tier::Requiem
        && fissure.mission_key == MissionType::Disruption
        && fissure.is_hard
}
