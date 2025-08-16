use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use indoc::formatdoc;
use itertools::Itertools;
use poise::{
    command,
    serenity_prelude::{
        CreateEmbed,
        CreateEmbedAuthor,
        FormattedTimestamp,
        FormattedTimestampStyle,
        colours::roles::DARK_RED,
    },
};
use poise_paginator::{CancellationType, paginate};
use warframe::market::{Item, Language, Status, queryable::OrderWithUser};

use crate::{
    CmdRet,
    Context,
    Error,
    commands::market::{I18nEn, find_best_matches, market_url, profile_url},
    emojis,
    utils::{self, ApplyIf},
};

async fn generate_order_embed(
    _ctx: Context<'_>,
    idx: usize,
    _cancellation_type: CancellationType,
    (orders, item): (Arc<[OrderWithUser]>, Arc<Item>),
) -> Result<CreateEmbed, Error> {
    let order = &orders[idx].order;
    let user = &orders[idx].user;
    let item_name = item.i18n_en().name.as_str();

    let embed = utils::embed()
        .author(
            CreateEmbedAuthor::new(&user.ingame_name).icon_url(
                user.avatar
                    .as_deref()
                    .map(|avatar| format!("https://warframe.market/static/assets/{avatar}"))
                    .unwrap_or_else(|| {
                        "https://warframe.market/static/assets/user/default-avatar.png".to_owned()
                    })
                    .to_owned(),
            ),
        )
        .title(item_name)
        .url(market_url(&item.slug))
        .field(
            "Price",
            format!("**`{}`** {}", order.platinum, emojis::PLATINUM),
            true,
        )
        .field("Quantity", format!("**`{}`**", order.quantity), true)
        .apply_optionally(order.rank, |embed, rank| {
            embed.field("Rank", format!("**`{rank}`**"), true)
        })
        .field(
            "Last Updated",
            FormattedTimestamp::new(
                order
                    .updated_at
                    .parse::<DateTime<Utc>>()
                    .expect("`order.updated_at` should be a valid ISO 8601 (UTC) timestamp")
                    .into(),
                Some(FormattedTimestampStyle::ShortDateTime),
            )
            .to_string(),
            false,
        )
        .field(
            "Buy",
            format!(
                "```\n/w {} Hi! I want to buy: \"{}\" for {} platinum. (warframe.market)```",
                user.ingame_name, item_name, order.platinum
            ),
            true,
        )
        .field(
            format!("{}'s Reputation", user.ingame_name),
            format!("**`{}`**", user.reputation),
            false,
        )
        .field(
            format!("{}'s WFM Profile", user.ingame_name),
            format!("[Profile]({})", profile_url(&user.ingame_name)),
            true,
        )
        .apply_if(user.status != Status::Ingame, |embed| embed.color(DARK_RED));

    Ok(embed)
}

/// Get a list of SELL orders for a specific item.
#[command(slash_command)]
pub async fn orders(
    ctx: Context<'_>,

    #[description = "The item to get orders for"]
    #[autocomplete = find_best_matches]
    #[rename = "item"]
    item_slug: String,

    #[description = "The rank of the item to filter by."]
    #[rename = "rank"]
    rank: Option<u8>,

    #[description = "The maximum number of orders to return. Defaults to 20."]
    #[rename = "limit"]
    limit: Option<usize>,

    #[description = "Whether to only include users that are currently ingame. Defaults to true."]
    #[rename = "ingame_only"]
    ingame_only: Option<bool>,
) -> CmdRet {
    let limit = limit.unwrap_or(20);
    let ingame_only = ingame_only.unwrap_or(true);
    let rank = rank.unwrap_or(0);

    let market = ctx.data().market();

    let Some(orders_with_user) = market
        .fetch_orders_by_slug(&item_slug, Language::En)
        .await?
    else {
        return Err("Item not found".into());
    };

    let orders = orders_with_user
        .into_iter()
        .filter(|order| {
            order.order.r#type == "sell"
                && (!ingame_only || order.user.status == Status::Ingame)
                && order.order.rank.unwrap_or(0) == rank
        })
        .sorted_by(|a, b| {
            a.order
                .platinum
                .cmp(&b.order.platinum)
                .then_with(|| b.order.updated_at.cmp(&a.order.updated_at))
        })
        .take(limit)
        .collect::<Arc<[_]>>();

    if orders.is_empty() {
        return Err(formatdoc!(
            "
        **No orders found for this item.**

        Ingame Only: `{ingame_only}`
        Rank: `{rank}`
        "
        )
        .into());
    }

    let item = market
        .fetch_item(&item_slug)
        .await?
        .expect("Item should be found");

    paginate(
        ctx,
        generate_order_embed,
        orders.len(),
        Duration::from_secs(60),
        (orders, Arc::new(item)),
    )
    .await?;

    Ok(())
}
