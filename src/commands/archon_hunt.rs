use chrono::{DateTime, Utc};
use indoc::formatdoc;
use itertools::Itertools;
use poise::{
    command,
    serenity_prelude::{CreateEmbed, FormattedTimestampStyle},
    CreateReply,
};
use warframe::worldstate::{queryable::ArchonHunt, ArchonHuntMission, TimedEvent};

use crate::{
    utils::{embed, to_timestamp},
    CmdRet,
    Context,
};

#[derive(strum::Display)]
enum ArchonShard {
    #[strum(to_string = "<:shard_amber:1299400892357017610> **Yellow Archon Shard**")]
    Yellow,
    #[strum(to_string = "<:shard_crimson:1299400724052054098> **Red Archon Shard**")]
    Red,
    #[strum(to_string = "<:shard_azure:1299400931825418271> **Blue Archon Shard**")]
    Blue,
}

impl ArchonShard {
    fn from_boss_str(boss: &str) -> Self {
        match boss {
            "Archon Amar" => Self::Red,
            "Archon Nira" => Self::Yellow,
            "Archon Boreal" => Self::Blue,
            _ => unreachable!("there are only 3 archons, which are covered here"),
        }
    }
}

#[command(slash_command, rename = "archon-hunt")]
pub async fn archon_hunt(ctx: Context<'_>) -> CmdRet {
    let archon_hunt = ctx.data().client.fetch::<ArchonHunt>().await?;
    let missions = &archon_hunt.missions;
    let obtainable_shard = ArchonShard::from_boss_str(&archon_hunt.boss);

    let embed = create_archon_hunt_embed(
        obtainable_shard,
        &archon_hunt.boss,
        archon_hunt.expiry(),
        missions,
    );

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn create_archon_hunt_embed(
    shard: ArchonShard,
    boss: &str,
    expiry: DateTime<Utc>,
    missions: &[ArchonHuntMission],
) -> CreateEmbed {
    embed()
        .title(boss)
        .url(format!("https://warframe.fandom.com/wiki/{}", boss.split(' ').join("_")))
        .description(formatdoc!(
            "
            {shard} obtainable

            __**{}** on **{}**__
            Level: 130-135
            
            __**{}** on **{}**__
            Level: 135-140
            
            __**{}** on **{}**__
            Level: 145-150

            Ends {}",
            missions[0].type_key,
            missions[0].node_key,
            missions[1].type_key,
            missions[1].node_key,
            missions[2].type_key,
            missions[2].node_key,
            to_timestamp(expiry, FormattedTimestampStyle::RelativeTime).expect("timestamp should be correct")
        ))
        .thumbnail(match shard {
            ArchonShard::Yellow => "https://static.wikia.nocookie.net/warframe/images/4/4c/ArchonNira.png/revision/latest?cb=20220418152944",
            ArchonShard::Red => "https://static.wikia.nocookie.net/warframe/images/b/be/ArchonAmar.png/revision/latest?cb=20220418152803",
            ArchonShard::Blue => "https://static.wikia.nocookie.net/warframe/images/1/1c/ArchonBoreal.png/revision/latest?cb=20220418152901",
        })
}
