#![allow(unstable_name_collisions)]
use std::{fmt::Display, time::Duration};

use chrono::{DateTime, Utc};
use futures::StreamExt;
use itertools::Itertools;
use poise::{
    CreateReply,
    command,
    serenity_prelude::{
        ButtonStyle,
        ComponentInteractionCollector,
        CreateActionRow,
        CreateButton,
        CreateEmbed,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
        FormattedTimestamp,
        FormattedTimestampStyle,
        Timestamp,
    },
};
use poise_paginator::{CancellationType, paginate};
use warframe::worldstate::{
    Opposite,
    SyndicateMission,
    TimedEvent,
    queryable::{CambionDrift, Cetus, OrbVallis},
};

use crate::{CmdRet, Context, Error, utils::embed};

#[command(
    slash_command,
    subcommands("cetus", "orb_vallis", "cambion_drift"),
    subcommand_required
)]
pub async fn worldstate(_: Context<'_>) -> CmdRet {
    Ok(())
}

/// Retrieves the current state of Cetus
#[command(slash_command)]
pub async fn cetus(ctx: Context<'_>) -> CmdRet {
    let wf = ctx.data().worldstate();
    let worldstate_item = wf.fetch::<Cetus>().await?;

    let embed = create_worldstate_embed(
        "Cetus",
        worldstate_item.state,
        worldstate_item.expiry(),
        "https://wiki.warframe.com/images/thumb/Plains_of_Eidolon.png/300px-Plains_of_Eidolon.png?c7c8c",
    )?;

    send_worldstate(ctx, embed, "Ostrons", "Ostron Bounties").await?;

    Ok(())
}

/// Retrieves the current state of the Orb Vallis
#[command(slash_command, rename = "orb-vallis")]
pub async fn orb_vallis(ctx: Context<'_>) -> CmdRet {
    let wf = ctx.data().worldstate();
    let worldstate_item = wf.fetch::<OrbVallis>().await?;

    let embed = create_worldstate_embed(
        "Orb Vallis",
        worldstate_item.state,
        worldstate_item.expiry(),
        "https://wiki.warframe.com/images/thumb/Orb_Vallis.png/300px-Orb_Vallis.png?7f8e7",
    )?;

    send_worldstate(ctx, embed, "Solaris United", "Solaris United Bounties").await?;

    Ok(())
}

/// Retrieves the current state of the Cambion Drift
#[command(slash_command, rename = "cambion-drift")]
pub async fn cambion_drift(ctx: Context<'_>) -> CmdRet {
    let wf = ctx.data().worldstate();
    let worldstate_item = wf.fetch::<CambionDrift>().await?;

    let embed = create_worldstate_embed(
        "Cambion Drift",
        worldstate_item.state,
        worldstate_item.expiry(),
        "https://wiki.warframe.com/images/thumb/CambionDrift.jpg/300px-CambionDrift.jpg?f2516",
    )?;

    send_worldstate(ctx, embed, "Entrati", "Entrati Bounties").await?;

    Ok(())
}

async fn send_worldstate(
    ctx: Context<'_>,
    worldstate_embed: CreateEmbed,
    syndicate_key: &'static str,
    bounty_title: &'static str,
) -> Result<(), Error> {
    let id = format!("{}_bounty", ctx.id());

    let bounty_button = CreateButton::new(&id)
        .label("Bounties")
        .style(ButtonStyle::Primary);

    let msg = ctx
        .send(
            CreateReply::default()
                .embed(worldstate_embed.clone())
                .components(vec![CreateActionRow::Buttons(vec![bounty_button])]),
        )
        .await?;

    let mut collector = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(Duration::from_secs(5))
        .filter(move |i| i.data.custom_id == id)
        .stream();

    if let Some(press) = collector.next().await {
        let mission = ctx
            .data()
            .worldstate()
            .fetch::<SyndicateMission>()
            .await?
            .into_iter()
            .find(|bounty: &SyndicateMission| bounty.syndicate_key == syndicate_key)
            .ok_or("Not found")?;

        let state = BountyState {
            mission: &mission,
            title: bounty_title,
        };

        press
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().components(Vec::new()),
                ),
            )
            .await?;

        paginate(
            ctx,
            generate_bounty_embed,
            mission.jobs.len(),
            Duration::from_secs(180),
            state,
        )
        .await?;
    } else {
        let disabled_bounty_button = CreateButton::new("_") // empty string cause it won't be clickable anyway
            .label("Bounties")
            .style(ButtonStyle::Primary)
            .disabled(true);

        msg.edit(
            ctx.into(),
            CreateReply::default()
                .embed(worldstate_embed)
                .components(vec![CreateActionRow::Buttons(vec![disabled_bounty_button])]),
        )
        .await?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct BountyState<'a> {
    mission: &'a SyndicateMission,
    title: &'static str,
}

fn create_worldstate_embed<S: Display + Opposite>(
    title: &str,
    region_state: S,
    expiry: DateTime<Utc>,
    thumbnail: &str,
) -> Result<CreateEmbed, Error> {
    Ok(embed()
        .title(title)
        .field("State", region_state.to_string(), false)
        .field(
            region_state.opposite().to_string(),
            FormattedTimestamp::new(
                Timestamp::from_unix_timestamp(expiry.timestamp())?,
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string(),
            false,
        )
        .thumbnail(thumbnail))
}

async fn generate_bounty_embed(
    _ctx: Context<'_>,
    idx: usize,
    _cancellation_type: CancellationType,
    state: BountyState<'_>,
) -> Result<CreateEmbed, Error> {
    let job = &state.mission.jobs[idx];

    let embed = embed()
        .title(state.title)
        .field(
            "Reward Pool",
            job.reward_pool
                .iter()
                .map(|reward| format!("- {reward}"))
                .join("\n"),
            false,
        )
        .field(
            "Ends",
            FormattedTimestamp::new(
                Timestamp::from_unix_timestamp(state.mission.expiry().timestamp())?,
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string(),
            false,
        )
        .field(
            "Standing per Stage",
            job.standing_stages
                .iter()
                .enumerate()
                .fold(String::new(), |acc, (stage_counter, &standing)| {
                    acc + &format!("{stage_counter}. `{standing}`\n")
                }),
            true,
        )
        .field(
            "Enemy Levels",
            job.enemy_levels
                .iter()
                .map(|num| num.to_string())
                .intersperse(" - ".to_owned())
                .collect::<String>(),
            true,
        );

    Ok(embed)
}
