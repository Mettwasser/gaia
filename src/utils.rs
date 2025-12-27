use std::future::Future;

use chrono::{DateTime, Utc};
use poise::serenity_prelude::{
    CreateEmbed,
    FormattedTimestamp,
    FormattedTimestampStyle,
    Timestamp,
    model::timestamp::InvalidTimestamp,
};
use sqlx::SqlitePool;

use crate::{
    Context,
    DEFAULT_COLOR,
    notifier::model::{RoleIdToMention, ServerSubscription, SubscriptionType},
};

pub fn to_timestamp(
    date: DateTime<Utc>,
    style: FormattedTimestampStyle,
) -> Result<FormattedTimestamp, InvalidTimestamp> {
    let timestamp = Timestamp::from_unix_timestamp(date.timestamp())?;
    Ok(FormattedTimestamp::new(timestamp, Some(style)))
}

/// A utility function to easily get the "default embed" for the bot
pub fn embed() -> CreateEmbed {
    CreateEmbed::default().color(DEFAULT_COLOR)
}

pub trait ApplyIf: Sized {
    fn apply_if<F, T>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> T,
        T: Into<Self>;

    fn apply_optionally<F, T, U>(self, optional: Option<T>, f: F) -> Self
    where
        F: FnOnce(Self, T) -> U,
        U: Into<Self>;
}

impl<T> ApplyIf for T {
    fn apply_if<F, U>(self, condition: bool, f: F) -> Self
    where
        F: FnOnce(Self) -> U,
        U: Into<Self>,
    {
        if condition { f(self).into() } else { self }
    }

    fn apply_optionally<F, I, U>(self, optional: Option<I>, f: F) -> Self
    where
        F: FnOnce(Self, I) -> U,
        U: Into<Self>,
    {
        match optional {
            Some(value) => f(self, value).into(),
            None => self,
        }
    }
}

pub trait ContextExt {
    fn db(&self) -> &SqlitePool;
    fn db_owned(&self) -> SqlitePool;
}

impl ContextExt for Context<'_> {
    fn db(&self) -> &SqlitePool {
        self.data().db()
    }

    fn db_owned(&self) -> SqlitePool {
        self.data().db_owned()
    }
}

pub trait DbExtension {
    fn insert_or_update_subscription(
        &self,
        subscription: SubscriptionType,
        guild_id: i64,
        channel_id: i64,
        role_id_to_mention: Option<i64>,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send;

    fn delete_subscription(
        &self,
        subscription: SubscriptionType,
        guild_id: i64,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send;

    fn get_subscriptions(
        &self,
        subscription: SubscriptionType,
    ) -> impl Future<Output = Result<Vec<ServerSubscription>, sqlx::Error>> + Send;

    fn get_subscriptions_for_guild(
        &self,
        guild_id: i64,
    ) -> impl Future<Output = Result<Vec<ServerSubscription>, sqlx::Error>> + Send;

    fn delete_all_subscriptions(
        &self,
        guild_id: i64,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send;

    fn delete_all_by_channel(
        &self,
        channel_id: i64,
    ) -> impl Future<Output = Result<(), sqlx::Error>> + Send;
}

impl DbExtension for SqlitePool {
    async fn insert_or_update_subscription(
        &self,
        subscription: SubscriptionType,
        guild_id: i64,
        channel_id: i64,
        role_id_to_mention: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.begin().await?;

        sqlx::query!(
            "
            INSERT INTO server_subscriptions (
                server_id,
                notification_channel_id,
                subscription_type,
                role_id_to_mention
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (server_id, subscription_type)
            DO UPDATE
            SET modified_at = CURRENT_TIMESTAMP,
                notification_channel_id = $2,
                subscription_type = $3,
                role_id_to_mention = $4;
            ",
            guild_id,
            channel_id,
            subscription,
            role_id_to_mention
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn delete_subscription(
        &self,
        subscription: SubscriptionType,
        guild: i64,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.begin().await?;

        sqlx::query!(
            "
            DELETE FROM server_subscriptions
            WHERE server_id = $1
                AND subscription_type = $2;
            ",
            guild,
            subscription
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_subscriptions(
        &self,
        subscription_type: SubscriptionType,
    ) -> Result<Vec<ServerSubscription>, sqlx::Error> {
        sqlx::query_as!(
            ServerSubscription,
            r#"
            SELECT 
                server_id,
                notification_channel_id,
                role_id_to_mention as "role_id_to_mention: RoleIdToMention",
                subscription_type as "subscription_type: SubscriptionType",
                created_at as "created_at: chrono::DateTime<Utc>",
                modified_at as "modified_at: chrono::DateTime<Utc>"
            FROM server_subscriptions
            WHERE subscription_type = $1
            "#,
            subscription_type
        )
        .fetch_all(self)
        .await
    }

    async fn get_subscriptions_for_guild(
        &self,
        guild_id: i64,
    ) -> Result<Vec<ServerSubscription>, sqlx::Error> {
        sqlx::query_as!(
            ServerSubscription,
            r#"
            SELECT 
                server_id,
                notification_channel_id,
                role_id_to_mention as "role_id_to_mention: RoleIdToMention",
                subscription_type as "subscription_type: SubscriptionType",
                created_at as "created_at: chrono::DateTime<Utc>",
                modified_at as "modified_at: chrono::DateTime<Utc>"
            FROM server_subscriptions
            WHERE server_id = $1
            "#,
            guild_id
        )
        .fetch_all(self)
        .await
    }

    async fn delete_all_subscriptions(&self, guild_id: i64) -> Result<(), sqlx::Error> {
        let mut tx = self.begin().await?;

        sqlx::query!(
            "
            DELETE FROM server_subscriptions
            WHERE server_id = $1;
            ",
            guild_id
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn delete_all_by_channel(&self, channel_id: i64) -> Result<(), sqlx::Error> {
        let mut tx = self.begin().await?;

        sqlx::query!(
            "
            DELETE FROM server_subscriptions
            WHERE notification_channel_id = $1;
            ",
            channel_id
        )
        .execute(tx.as_mut())
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
