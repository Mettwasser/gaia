use std::cmp::Reverse;

use poise::{CreateReply, command, serenity_prelude::AutocompleteChoice};
use reqwest::StatusCode;
use strsim::jaro_winkler;
use warframe::market::{Language, Slug};

use crate::{CmdRet, Context, Error, utils};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct StatisticInfo {
    average: f64,
    moving_avg: Option<f64>,
    amount_sold: u32,
}

impl From<Vec<Statistic>> for StatisticInfo {
    fn from(statistics: Vec<Statistic>) -> Self {
        let average = statistics.last().unwrap().avg_price;

        let moving_avg = statistics.last().unwrap().moving_avg;

        let amount_sold = statistics.iter().map(|stat| stat.volume).sum::<u32>();

        Self {
            average,
            moving_avg,
            amount_sold,
        }
    }
}

async fn find_best_matches(ctx: Context<'_>, query: &str) -> Vec<AutocompleteChoice> {
    // Get all candidate items first
    let candidates = ctx.data().market().items(Language::En).await.unwrap();

    // 1. Map each candidate to a tuple containing its score and a reference to it
    let mut scored_candidates: Vec<_> = candidates
        .iter()
        .map(|candidate| {
            let name = &candidate.i18n.get(&Language::En).unwrap().name;
            let score = (jaro_winkler(query, name) * 1000.0) as i32;
            (score, candidate)
        })
        .collect();

    // 2. Sort the list by score in descending order.
    // We use `std::cmp::Reverse` on the key (the score) for an efficient descending sort.
    scored_candidates.sort_by_key(|(score, _)| Reverse(*score));

    // 3. Take the top 25 and map them to the desired output format
    scored_candidates
        .into_iter()
        .take(25)
        .map(|(_score, candidate)| {
            AutocompleteChoice::new(
                &candidate.i18n.get(&Language::En).unwrap().name,
                candidate.slug.clone(),
            )
        })
        .collect::<Vec<_>>()
}

async fn get_statistics(ctx: Context<'_>, item_slug: &str) -> Result<Statistics, Error> {
    let url = format!("https://api.warframe.market/v1/items/{item_slug}/statistics");

    if let Some(statistics) = ctx.data().market_statistic_cache().get(&url).await {
        return Ok(statistics);
    }

    let response = reqwest::get(&url).await?;

    if response.status() == StatusCode::NOT_FOUND {
        return Err("Item not found".into());
    }

    let statistics = response.json::<Statistics>().await?;

    ctx.data()
        .market_statistic_cache()
        .insert(url, statistics.clone())
        .await;

    Ok(statistics)
}

fn filter_statistic(
    statistics: Vec<Statistic>,
    mod_rank: Option<i32>,
    has_mod_rank: bool,
) -> Vec<Statistic> {
    if let Some(rank) = mod_rank
        && has_mod_rank
    {
        statistics
            .into_iter()
            .filter(|stat| stat.mod_rank.unwrap() == rank as u8)
            .collect::<Vec<_>>()
    } else if has_mod_rank {
        statistics
            .into_iter()
            .filter(|stat| stat.mod_rank.unwrap() == 0)
            .collect::<Vec<_>>()
    } else {
        statistics
    }
}

#[command(slash_command)]
pub async fn average(
    ctx: Context<'_>,
    #[description = "The item to to get the average price for"]
    #[autocomplete = find_best_matches]
    item_slug: String,
    #[description = "Mod Rank of the item, if applicable. Defaults to 0."] mod_rank: Option<i32>,
) -> CmdRet {
    let statistics = get_statistics(ctx, &item_slug)
        .await?
        .payload
        .statistics_closed
        .the_48_hours;

    let market = ctx.data().market();

    let item = market
        .fetch_item(&Slug::new_unchecked(&item_slug))
        .await?
        .expect("Item should be found");

    let item_name = item.i18n.get(&Language::En).unwrap().name.as_str();

    if statistics.is_empty() {
        ctx.send(
            CreateReply::default().embed(
                utils::embed().description(format!("No statistics found for `{item_name}`")),
            ),
        )
        .await?;

        return Ok(());
    }

    let has_mod_rank = item.max_rank.is_some();

    // filter mod rank so for example R10s don't inflate the average price
    let statistics = filter_statistic(statistics, mod_rank, has_mod_rank);

    let statistic_info = StatisticInfo::from(statistics);

    ctx.send(
        CreateReply::default().embed(
            utils::embed()
                .title(format!(
                    "{}{}",
                    item_name,
                    if has_mod_rank {
                        format!(" R{}", mod_rank.unwrap_or(0))
                    } else {
                        "".to_owned()
                    }
                ))
                .url(format!("https://warframe.market/items/{item_slug}"))
                .field(
                    "Average",
                    format!(
                        "**`{}`** <:platinum:1405547068704952351>",
                        statistic_info.average
                    ),
                    false,
                )
                .field(
                    "Moving Average",
                    format!(
                        "**`{}`** <:platinum:1405547068704952351>",
                        statistic_info
                            .moving_avg
                            .map(|avg| avg.to_string())
                            .unwrap_or_else(|| "N/A".into())
                    ),
                    false,
                )
                .field(
                    "Sales (Last 48 hours)",
                    format!("**`{}`**", statistic_info.amount_sold),
                    false,
                )
                .thumbnail(format!(
                    "https://warframe.market/static/assets/{}",
                    item.i18n.get(&Language::En).unwrap().icon
                )),
        ),
    )
    .await?;

    Ok(())
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistics {
    pub payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub statistics_closed: StatisticsClosed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticsClosed {
    #[serde(rename = "48hours")]
    pub the_48_hours: Vec<Statistic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistic {
    pub datetime: String,

    pub volume: u32,

    pub closed_price: i64,

    pub avg_price: f64,

    pub moving_avg: Option<f64>,

    pub mod_rank: Option<u8>,
}
