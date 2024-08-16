#![allow(unstable_name_collisions)]
use std::{fmt::Display, time::Duration};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use poise::{
    command,
    serenity_prelude::{
        ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
        CreateInteractionResponse, CreateInteractionResponseMessage, FormattedTimestamp,
        FormattedTimestampStyle, ReactionType, Timestamp,
    },
    CreateReply, ReplyHandle,
};
use warframe::worldstate::models::{
    CambionDrift, Cetus, Opposite, OrbVallis, SyndicateJob, SyndicateMission, TimedEvent,
};

use crate::{paginate::Paginate, CmdRet, Context, Error, DEFAULT_COLOR};

/// Retrieves the current state of Cetus
#[command(slash_command)]
pub async fn cetus(ctx: Context<'_>) -> CmdRet {
    crate::send_worldstate_response!(ctx, Cetus, "Cetus", "Ostrons", "Ostron Bounties");
    Ok(())
}

/// Retrieves the current state of the Orb Vallis
#[command(slash_command, rename = "orb-vallis")]
pub async fn orb_valis(ctx: Context<'_>) -> CmdRet {
    crate::send_worldstate_response!(
        ctx,
        OrbVallis,
        "Orb Vallis",
        "Solaris United",
        "Solaris United Bounties"
    );
    Ok(())
}

/// Retrieves the current state of the Cambion Drift
#[command(slash_command, rename = "cambion-drift")]
pub async fn cambion_drift(ctx: Context<'_>) -> CmdRet {
    crate::send_worldstate_response!(
        ctx,
        CambionDrift,
        "Cambion Drift",
        "Entrati",
        "Entrati Bounties"
    );
    Ok(())
}

fn create_worldstate_embed<S: Display + Opposite>(
    title: &'static str,
    region_state: S,
    expiry: DateTime<Utc>,
    thumbnail: &'static str,
) -> Result<CreateEmbed, Error> {
    Ok(CreateEmbed::new()
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
        .thumbnail(thumbnail)
        .color(DEFAULT_COLOR))
}

#[macro_export]
macro_rules! send_worldstate_response {
    ($ctx:expr, $type:ty, $title:literal, $faction_name:literal, $bounty_title:literal) => {
        let wf = $ctx.data().client();
        let worldstate_item = wf.fetch::<$type>().await?;

        let embed = create_worldstate_embed($title, worldstate_item.state.clone(), worldstate_item.expiry(), "https://www-static.warframe.com/uploads/thumbnails/60e6a05ebbf14112d96bc03b2bfe5a8c_1600x900.png")?;

        let bounty_button = CreateButton::new(format!("{}_bounty", $ctx.id()))
            .label("Bounties")
            .style(ButtonStyle::Primary);

        let msg = $ctx.send(
            CreateReply::default()
                .embed(embed.clone())
                .components(vec![CreateActionRow::Buttons(vec![bounty_button])]),
        )
            .await?;

        do_bounty_pagination($ctx, msg, $faction_name, $bounty_title, embed).await?;
    };
}
fn create_bounty_embeds(
    title: &str,
    jobs: Vec<SyndicateJob>,
    expiry: DateTime<Utc>,
) -> Result<Vec<CreateEmbed>, Error> {
    let mut embeds = Vec::new();
    for job in jobs {
        embeds.push(
            CreateEmbed::new()
                .color(DEFAULT_COLOR)
                .title(title)
                .description(format!(
                    "**Reward Pool**\n```\n{}\n```",
                    job.reward_pool.join("\n")
                ))
                .field(
                    "Ends",
                    FormattedTimestamp::new(
                        Timestamp::from_unix_timestamp(expiry.timestamp())?,
                        Some(FormattedTimestampStyle::RelativeTime),
                    )
                    .to_string(),
                    false,
                )
                .field(
                    "Standing per Stage",
                    job.standing_stages.iter().enumerate().fold(
                        String::new(),
                        |acc, (stage_counter, &standing)| {
                            acc + &format!("{}. `{}`\n", stage_counter, standing)
                        },
                    ),
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
                ),
        )
    }
    Ok(embeds)
}

fn get_paginate_components(
    left_enabled: bool,
    right_enabled: bool,
    current_index: usize,
    total_size: usize,
    ids: &[&str; 5],
) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![
        CreateButton::new(ids[0])
            .emoji(ReactionType::Unicode("⏪".to_owned()))
            .style(ButtonStyle::Success)
            .disabled(!left_enabled),
        CreateButton::new(ids[1])
            .emoji(ReactionType::Unicode("◀️".to_owned()))
            .style(ButtonStyle::Primary)
            .disabled(!left_enabled),
        CreateButton::new(ids[2])
            .label(format!("{} / {}", current_index + 1, total_size))
            .disabled(true),
        CreateButton::new(ids[3])
            .emoji(ReactionType::Unicode("▶️".to_owned()))
            .style(ButtonStyle::Primary)
            .disabled(!right_enabled),
        CreateButton::new(ids[4])
            .emoji(ReactionType::Unicode("⏩".to_owned()))
            .style(ButtonStyle::Success)
            .disabled(!right_enabled),
    ])]
}

async fn do_bounty_pagination(
    ctx: Context<'_>,
    original_message: ReplyHandle<'_>,
    syndicate_key: &'static str,
    header: &'static str,
    initial_worldstate_embed: CreateEmbed,
) -> CmdRet {
    let interaction_id = format!("{}_bounty", ctx.id());
    if let Some(bounty_press) = ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(Duration::from_secs(5))
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .filter(move |mci| mci.data.custom_id == interaction_id)
        .await
    {
        let bounty: SyndicateMission = ctx
            .data()
            .client()
            .fetch::<SyndicateMission>()
            .await?
            .into_iter()
            .find(|bounty: &SyndicateMission| bounty.syndicate_key == syndicate_key)
            .ok_or("Not found")?;

        let fast_rewind_id = format!("{}_fast_rewind", ctx.id());
        let rewind_id = format!("{}_rewind", ctx.id());
        let counter_id = format!("{}_counter", ctx.id());
        let forward_id = format!("{}_forward", ctx.id());
        let fast_forward_id = format!("{}_fast_forward", ctx.id());
        let ids = &[
            fast_rewind_id.as_str(),
            rewind_id.as_str(),
            counter_id.as_str(),
            forward_id.as_str(),
            fast_forward_id.as_str(),
        ];

        let components = get_paginate_components(false, true, 0, bounty.jobs.len(), ids);

        let expiry = bounty.expiry();
        let embeds = create_bounty_embeds(header, bounty.jobs, expiry)?;
        let first_embed = embeds.first().ok_or("Not Found!")?;

        bounty_press
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .embed(first_embed.clone())
                        .components(components),
                ),
            )
            .await?;

        let mut paginator = Paginate::new(&embeds);

        let ctx_id = ctx.id();

        while let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context())
            .timeout(Duration::from_secs(5))
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .filter(move |interaction| interaction.data.custom_id.starts_with(&ctx_id.to_string()))
            .await
        {
            let next_embed: Option<&CreateEmbed> = if press.data.custom_id == fast_rewind_id {
                paginator.first_page()
            } else if press.data.custom_id == rewind_id {
                paginator.previous_page()
            } else if press.data.custom_id == forward_id {
                paginator.next_page()
            } else if press.data.custom_id == fast_forward_id {
                paginator.last_page()
            } else {
                panic!()
            };

            if let Some(embed) = next_embed {
                let mut create_reply = CreateInteractionResponseMessage::new();

                let (left, right) = if paginator.current_idx() == 0 {
                    (false, true)
                } else if paginator.current_idx() == embeds.len() - 1 {
                    (true, false)
                } else {
                    (true, true)
                };

                create_reply = create_reply.components(get_paginate_components(
                    left,
                    right,
                    paginator.current_idx(),
                    embeds.len(),
                    ids,
                ));

                press
                    .create_response(
                        ctx,
                        CreateInteractionResponse::UpdateMessage(create_reply.embed(embed.clone())),
                    )
                    .await?;
            } else {
                press
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
            }
        }

        original_message
            .edit(
                ctx,
                CreateReply::default()
                    .components(get_paginate_components(
                        false,
                        false,
                        paginator.current_idx(),
                        embeds.len(),
                        ids,
                    ))
                    .embed(
                        embeds
                            .get(paginator.current_idx())
                            .ok_or("Couldn't Paginate this Page")?
                            .to_owned(),
                    ),
            )
            .await?;
        return Ok(());
    }

    original_message
        .edit(
            ctx,
            CreateReply::default()
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    format!("{}_bounty", ctx.id()),
                )
                .label("Bounties")
                .style(ButtonStyle::Primary)
                .disabled(true)])])
                .embed(initial_worldstate_embed),
        )
        .await?;

    Ok(())
}
