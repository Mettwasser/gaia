use poise::command;

use crate::{CmdRet, Context};

pub mod average;

use average::average;

#[command(slash_command, subcommands("average"))]
pub async fn market(_ctx: Context<'_>) -> CmdRet {
    Ok(())
}
