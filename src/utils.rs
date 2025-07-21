use chrono::{
    DateTime,
    Utc,
};
use poise::serenity_prelude::{
    model::timestamp::InvalidTimestamp,
    CreateEmbed,
    FormattedTimestamp,
    FormattedTimestampStyle,
    Timestamp,
};

use crate::DEFAULT_COLOR;

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
