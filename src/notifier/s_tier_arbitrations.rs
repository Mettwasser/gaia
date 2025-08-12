use std::sync::Arc;

use arbitration_data::model::mapped::ArbitrationInfo;
use chrono::Utc;
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

use crate::{
    notifier::{model::SubscriptionType, Notifier},
    utils::{self, ApplyIf, DbExtension},
    AppData,
    Error,
};

fn build_embed(arbi: &ArbitrationInfo) -> CreateEmbed {
    utils::embed()
        .title("New S-Tier Arbitration")
        .field("Node", format!("{} ({})", &arbi.node, &arbi.planet), true)
        .field("Mission Type", &arbi.mission_type, true)
        .field(
            "Ends",
            FormattedTimestamp::new(
                arbi.expiry.into(),
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string(),
            false,
        )
        .timestamp(Timestamp::now())
}

pub struct STierArbitrationListener;

impl Notifier for STierArbitrationListener {
    async fn run(ctx: serenity_prelude::Context, data: Arc<AppData>) -> Result<(), Error> {
        while let Ok(next_arbi) = data.arbi_data().upcoming_by_tier(arbitration_data::Tier::S) {
            if next_arbi.activation > Utc::now() {
                tracing::info!(time_to_sleep = ?(next_arbi.activation - Utc::now()).to_std()?, upcoming_arbi = ?next_arbi);
                tokio::time::sleep((next_arbi.activation - Utc::now()).to_std()?).await;
            }

            let subscriptions = data
                .db()
                .get_subscriptions(SubscriptionType::STierArbitrations)
                .await?;

            let embed = build_embed(next_arbi);

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
                            "Failed to send notification for S-Tier Arbitration",
                        );
                    }

                    result
                })
                .collect::<Vec<_>>();

            join_all(notification_tasks).await;
        }

        Ok(())
    }
}
