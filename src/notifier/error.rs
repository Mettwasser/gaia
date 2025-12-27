use poise::serenity_prelude::{self, ChannelId, ModelError};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{AppData, Error, utils::DbExtension};

#[derive(Debug)]
pub struct NotifierError {
    pub channel_id: ChannelId,
    pub error: Error,
}

impl NotifierError {
    pub fn new(channel_id: ChannelId, error: Error) -> Self {
        Self { channel_id, error }
    }
}

pub async fn handle_notifier_error(mut rx: UnboundedReceiver<NotifierError>, data: AppData) {
    while let Some(NotifierError {
        channel_id,
        error: err,
    }) = rx.recv().await
    {
        tracing::error!(error = %err, "Eidolon Hunts notifier error");

        if let Some(err) = err.downcast_ref::<serenity_prelude::Error>()
            && (matches!(
                err,
                serenity_prelude::Error::Model(ModelError::GuildNotFound)
            ) || matches!(
                err,
                serenity_prelude::Error::Model(ModelError::ChannelNotFound)
            ) || matches!(
                err,
                serenity_prelude::Error::Model(ModelError::InvalidPermissions { .. })
            ))
        {
            data.db()
                .delete_all_by_channel(channel_id.get() as i64)
                .await
                .unwrap_or_default();

            continue;
        }
    }
}
