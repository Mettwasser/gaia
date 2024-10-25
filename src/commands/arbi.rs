use arbitration_data::model::mapped::Tier;
use chrono::Duration;
use poise::{
    command,
    serenity_prelude::{CreateEmbed, FormattedTimestamp, FormattedTimestampStyle, Timestamp},
    ChoiceParameter,
    CreateReply,
};

use crate::{
    embed_paginator::{LazyEmbedPaginator, LazyPaginationTrait},
    CmdRet,
    Context,
    DEFAULT_COLOR,
};

#[derive(ChoiceParameter, derive_more::Display, Clone, Debug)]
pub enum UserArbitrationTier {
    #[name = "S Tier Map"]
    S,
    #[name = "A Tier Map"]
    A,
    #[name = "B Tier Map"]
    B,
    #[name = "C Tier Map"]
    C,
    #[name = "D Tier Map"]
    D,
    #[name = "F Tier Map"]
    F,
}

impl From<UserArbitrationTier> for Tier {
    fn from(value: UserArbitrationTier) -> Self {
        match value {
            UserArbitrationTier::S => Tier::S,
            UserArbitrationTier::A => Tier::A,
            UserArbitrationTier::B => Tier::B,
            UserArbitrationTier::C => Tier::C,
            UserArbitrationTier::D => Tier::D,
            UserArbitrationTier::F => Tier::F,
        }
    }
}

/// Shows you the upcoming Arbitration, optionally filtered by a Map Tier
#[command(slash_command, rename = "upcoming-arbitration")]
pub async fn upcoming_arbitration(
    ctx: Context<'_>,
    #[description = "The Tier of the Arbitration Map you want to look up"] tier: Option<
        UserArbitrationTier,
    >,
) -> CmdRet {
    let mut embed = CreateEmbed::default().color(DEFAULT_COLOR);
    let arbi_info = if let Some(tier) = tier {
        match ctx.data().arbi_data().upcoming_by_tier(tier.clone().into()) {
            Ok(data) => data,
            Err(_) => {
                ctx.say(format!(
                    "Could not find any upcoming {tier} Tier Arbitrations.",
                ))
                .await?;
                return Ok(());
            },
        }
    } else {
        match ctx.data().arbi_data().upcoming() {
            Ok(data) => data,
            Err(_) => {
                ctx.say("Could not find any upcoming Arbitrations.").await?;
                return Ok(());
            },
        }
    };

    embed = embed
        .title(format!("{} Tier Arbitration", arbi_info.tier))
        .field(
            "Node",
            format!(
                "[{}](https://youtu.be/8ybW48rKBME?si=3pbdudMX_CTUAJ8T)",
                &arbi_info.node
            ),
            true,
        )
        .field(
            "Planet",
            format!(
                "[{}](https://youtu.be/8ybW48rKBME?si=3pbdudMX_CTUAJ8T)",
                &arbi_info.planet
            ),
            true,
        )
        .field(
            "Mission Type",
            format!(
                "**{}**, against **{}**",
                &arbi_info.mission_type, arbi_info.faction
            ),
            false,
        )
        .field(
            "Starts",
            FormattedTimestamp::new(
                Timestamp::from_unix_timestamp(arbi_info.activation.timestamp())?,
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string(),
            true,
        )
        .field(
            "Ends",
            FormattedTimestamp::new(
                Timestamp::from_unix_timestamp(arbi_info.expiry.timestamp())?,
                Some(FormattedTimestampStyle::RelativeTime),
            )
            .to_string(),
            true,
        );

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

const AMOUNT_PER_PAGE: usize = 10;

#[derive(Clone)]
struct PaginationState<'a> {
    ctx: Context<'a>,
    tier: Option<UserArbitrationTier>,
}

impl<'a> LazyPaginationTrait<'a> for PaginationState<'a> {
    fn ctx(&self) -> Context<'a> {
        self.ctx
    }
}

async fn get_page(state: PaginationState<'_>, idx: usize) -> Option<CreateEmbed> {
    let arbi_data = &state.ctx.data().arbi_data;
    let mut description = String::new();
    description.push_str(&format!(
        "**`Tier  {:<15} {:<10} When`**\n",
        "Node", "Planet"
    ));

    let skip = AMOUNT_PER_PAGE * idx;
    let take = AMOUNT_PER_PAGE;
    for (key, value) in arbi_data
        .iter_upcoming()
        .filter(|(_k, v)| {
            state
                .tier
                .as_ref()
                .map(|tier| v.tier == tier.clone().into())
                .unwrap_or(true)
        })
        .skip(skip)
        .take(take)
    {
        description.push_str(&format!(
            "`[{}]   {:<15} {:<10}` {}\n",
            value.tier,
            value.node,
            value.planet,
            FormattedTimestamp::new(
                Timestamp::from_unix_timestamp(*key).unwrap(),
                Some(FormattedTimestampStyle::RelativeTime),
            )
        ))
    }

    if !description.is_empty() {
        let title = match state.tier {
            Some(tier) => format!("Upcoming {} Tier Arbitrations", tier),
            None => "Upcoming Arbitrations".to_owned(),
        };
        Some(
            CreateEmbed::new()
                .description(description)
                .title(title)
                .color(DEFAULT_COLOR),
        )
    } else {
        None
    }
}

/// Shows you all upcoming Arbitrations, optionally filtered by a Map Tier
#[command(slash_command, rename = "upcoming-arbitrations")]
pub async fn upcoming_arbitrations(
    ctx: Context<'_>,
    #[description = "The Tier of the Arbitration Map you want to look up"] tier: Option<
        UserArbitrationTier,
    >,
) -> CmdRet {
    let state = PaginationState { ctx, tier };

    let paginator_length = ctx
        .data()
        .arbi_data
        .iter_upcoming()
        .filter(|(_k, v)| {
            state
                .tier
                .as_ref()
                .map(|tier| v.tier == tier.clone().into())
                .unwrap_or(true)
        })
        .count()
        / AMOUNT_PER_PAGE;

    let mut paginator = LazyEmbedPaginator::new(get_page, paginator_length, state);
    paginator.start(Duration::minutes(2)).await?;

    Ok(())
}
