use std::cmp::Reverse;

use poise::{command, serenity_prelude::AutocompleteChoice};
use strsim::jaro_winkler;
use warframe::market::{Item, ItemI18N, ItemShort, ItemShortI18N, Language};

use crate::{CmdRet, Context};

pub mod average;
pub mod orders;

#[command(slash_command, subcommands("average::average", "orders::orders"))]
pub async fn market(_ctx: Context<'_>) -> CmdRet {
    Ok(())
}

async fn find_best_matches(ctx: Context<'_>, query: &str) -> Vec<AutocompleteChoice> {
    // Get all candidate items first
    let candidates = ctx.data().market().items(Language::En).await.unwrap();

    // 1. Map each candidate to a tuple containing its score and a reference to it
    let mut scored_candidates: Vec<_> = candidates
        .iter()
        .map(|candidate| {
            let name = &candidate.i18n_en().name;
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
            AutocompleteChoice::new(&candidate.i18n_en().name, candidate.slug.clone())
        })
        .collect::<Vec<_>>()
}

pub fn market_url(slug: &impl AsRef<str>) -> String {
    format!("https://warframe.market/items/{}", slug.as_ref())
}

pub fn profile_url(username: &str) -> String {
    format!("https://warframe.market/profile/{username}")
}

pub trait I18nEn<Item> {
    fn i18n_en(&self) -> &Item;
}

impl I18nEn<ItemI18N> for Item {
    fn i18n_en(&self) -> &ItemI18N {
        self.i18n.get(&Language::En).unwrap()
    }
}

impl I18nEn<ItemShortI18N> for ItemShort {
    fn i18n_en(&self) -> &ItemShortI18N {
        self.i18n.get(&Language::En).unwrap()
    }
}
