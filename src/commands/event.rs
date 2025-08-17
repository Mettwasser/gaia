use std::{sync::Arc, time::Duration};

use indoc::formatdoc;
use itertools::Itertools;
use poise::{CreateReply, command, serenity_prelude::CreateEmbed};
use poise_paginator::{CancellationType, paginate};
use warframe::worldstate::queryable::Event;

use crate::{CmdRet, Context, Error, emojis, utils};

async fn generate_embed(
    _ctx: Context<'_>,
    idx: usize,
    _cancellation_type: CancellationType,
    state: Arc<[CreateEmbed]>,
) -> Result<CreateEmbed, Error> {
    Ok(state[idx].clone())
}

#[command(slash_command)]
pub async fn events(ctx: Context<'_>) -> CmdRet {
    let events = ctx.data().worldstate().fetch::<Event>().await?;

    let embeds = events
        .into_iter()
        .map(|event| {
            utils::embed()
                .title(event.description.as_deref().unwrap_or("-"))
                .description(formatdoc! {
                    "
                    **Missions**
                    **`{}`**
                    **Total Rewards**
                    {}
                    ",
                    event.concurrent_nodes.len(),
                    event.rewards.into_iter()
                        .filter(|reward|
                            !reward.counted_items.is_empty()
                            || !reward.items.is_empty()
                            || reward.credits > 0
                        )
                        .map(|reward|
                            format!(
                                "- {}{}",
                                reward.item_string,
                                if reward.credits > 0 {
                                    format!(" + {} {}", reward.credits, emojis::CREDITS)
                                } else {
                                    String::new()
                                }
                        )
                    ).join("\n")
                })
        })
        .collect::<Arc<[_]>>();

    if embeds.is_empty() {
        return Err("No events found".into());
    }

    if embeds.len() == 1 {
        ctx.send(CreateReply::default().embed(embeds[0].clone()))
            .await?;
    } else {
        paginate(
            ctx,
            generate_embed,
            embeds.len(),
            Duration::from_secs(60),
            embeds,
        )
        .await?;
    }

    Ok(())
}
