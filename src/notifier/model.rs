use std::ops::Deref;

use chrono::Utc;
use derive_more::Display;
use poise::serenity_prelude::{ChannelId, GuildId, RoleId};
use sqlx::{error::BoxDynError, Decode, Sqlite};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, sqlx::Type, poise::ChoiceParameter,
)]
#[repr(i64)]
pub enum SubscriptionType {
    #[name = "S-Tier Arbitrations"]
    STierArbitrations,

    #[name = "Steel Path Disruption Fissures"]
    SteelPathDisruptionFissures,

    #[name = "Eidolon Hunts"]
    EidolonHunts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub struct ServerId(GuildId);

impl From<i64> for ServerId {
    fn from(id: i64) -> Self {
        Self(GuildId::new(id as u64))
    }
}

impl Deref for ServerId {
    type Target = GuildId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Display)]
pub struct NotificationChannelId(ChannelId);

impl From<i64> for NotificationChannelId {
    fn from(id: i64) -> Self {
        Self(ChannelId::new(id as u64))
    }
}

impl Deref for NotificationChannelId {
    type Target = ChannelId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
pub struct RoleIdToMention(RoleId);

impl Decode<'_, Sqlite> for RoleIdToMention {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'_>) -> Result<Self, BoxDynError> {
        let id: i64 = Decode::<'_, Sqlite>::decode(value)?;
        Ok(Self(RoleId::new(id as u64)))
    }
}

impl Deref for RoleIdToMention {
    type Target = RoleId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServerSubscription {
    pub server_id: ServerId,
    pub notification_channel_id: NotificationChannelId,
    pub subscription_type: SubscriptionType,
    pub created_at: chrono::DateTime<Utc>,
    pub modified_at: Option<chrono::DateTime<Utc>>,
    pub role_id_to_mention: Option<RoleIdToMention>,
}
