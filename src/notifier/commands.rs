use poise::{
    command,
    serenity_prelude::{GuildChannel, Mentionable, RoleId},
    ChoiceParameter,
    CreateReply,
};

use crate::{
    notifier::model::SubscriptionType,
    utils::{self, ContextExt, DbExtension},
    CmdRet,
    Context,
};

#[command(
    slash_command,
    subcommands("setup", "remove", "list"),
    subcommand_required,
    default_member_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn notifier(_: Context<'_>) -> CmdRet {
    Ok(())
}

#[command(slash_command)]
pub async fn setup(
    ctx: Context<'_>,
    // ---
    #[description = "The Channel to send notifications to."]
    #[channel_types("Text")]
    channel: GuildChannel,
    // ---
    #[description = "The type of notifications to subscribe to."] subscription: SubscriptionType,
    // ---
    #[description = "The role to mention when sending notifications for the specific subscription."]
    #[rename = "role_to_mention"]
    role_id_to_mention: Option<RoleId>,
) -> CmdRet {
    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let channel_id = channel.id.get() as i64;

    ctx.db()
        .insert_or_update_subscription(
            subscription,
            guild_id,
            channel_id,
            role_id_to_mention.map(|v| v.get() as i64),
        )
        .await?;

    ctx.send(
        CreateReply::default().reply(true).embed(
            utils::embed()
                .title("Setup Successful")
                .description(format!(
                    "You will now receive notifications for `{}` in {}.{}",
                    subscription.name(),
                    channel.mention(),
                    role_id_to_mention
                        .map(|v| format!("\nRole to mention upon notification: {}", v.mention()))
                        .unwrap_or_else(|| "".into())
                )),
        ),
    )
    .await?;

    Ok(())
}

#[command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The type of notifications to unsubscribe from."]
    subscription: SubscriptionType,
) -> CmdRet {
    let guild_id = ctx.guild_id().unwrap().get() as i64;

    ctx.db().delete_subscription(subscription, guild_id).await?;

    ctx.send(
        CreateReply::default().reply(true).embed(
            utils::embed()
                .title("Successfully Unsubscribed")
                .description(format!(
                    "You will no longer receive notifications for `{}` in this server.",
                    subscription.name()
                )),
        ),
    )
    .await?;

    Ok(())
}

#[command(slash_command)]
pub async fn list(ctx: Context<'_>) -> CmdRet {
    let guild_id = ctx.guild_id().unwrap().get() as i64;

    let subscriptions = ctx.db().get_subscriptions_for_guild(guild_id).await?;

    if subscriptions.is_empty() {
        ctx.send(
            CreateReply::default()
                .reply(true)
                .embed(utils::embed().description("You have no subscriptions in this server.")),
        )
        .await?;
        return Ok(());
    }

    ctx.send(
        CreateReply::default().reply(true).embed(
            utils::embed().title("Current Subscriptions").description(
                subscriptions
                    .iter()
                    .map(|s| {
                        format!(
                            "- `{}` in {}{}",
                            s.subscription_type.name(),
                            s.notification_channel_id.mention(),
                            s.role_id_to_mention
                                .map(|v| format!(" mentioning {}", v.mention()))
                                .unwrap_or_else(|| "".into())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        ),
    )
    .await?;

    Ok(())
}
