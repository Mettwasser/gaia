use std::sync::Arc;

use chrono::Utc;
use poise::serenity_prelude::{
    self,
    async_trait,
    CreateMessage,
    FormattedTimestamp,
    FormattedTimestampStyle,
    Mentionable,
    Timestamp,
};

use crate::{
    notifier::{model::SubscriptionType, Notifier},
    utils::{self, ApplyIf, DbExtension},
    Data,
    Error,
};

pub struct STierArbitrationListener;

#[async_trait]
impl Notifier for STierArbitrationListener {
    async fn run(&self, ctx: serenity_prelude::Context, data: Arc<Data>) -> Result<(), Error> {
        let db = data.db();
        let arbi_data = data.arbi_data();
        while let Ok(next_arbi) = arbi_data.upcoming_by_tier(arbitration_data::Tier::S) {
            if next_arbi.activation > Utc::now() {
                tracing::info!(time_to_sleep = ?(next_arbi.activation - Utc::now()).to_std()?, upcoming_arbi = ?next_arbi);
                tokio::time::sleep((next_arbi.activation - Utc::now()).to_std()?).await;
            }

            let subscriptions = db
                .get_subscriptions(SubscriptionType::STierArbitrations)
                .await?;

            for subscription in subscriptions {
                subscription
                    .notification_channel_id
                    .send_message(
                        &ctx,
                        CreateMessage::new()
                            .apply_optional(subscription.role_id_to_mention, |msg, role_id| {
                                msg.content(role_id.mention().to_string())
                            })
                            .add_embed(
                                utils::embed()
                                    .title("New S-Tier Arbitration")
                                    .field("Node", next_arbi.node.as_str(), true)
                                    .field("Planet", next_arbi.planet.as_str(), true)
                                    .field(
                                        "Ends",
                                        FormattedTimestamp::new(
                                            next_arbi.expiry.into(),
                                            Some(FormattedTimestampStyle::RelativeTime),
                                        )
                                        .to_string(),
                                        false,
                                    )
                                    .timestamp(Timestamp::now()),
                            ),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}
