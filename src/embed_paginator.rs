use {
    crate::{
        error_embed,
        paginate::{Paginate, PaginateLazily},
        CmdRet, Context, Error,
    },
    chrono::Duration,
    poise::{
        serenity_prelude::{
            ButtonStyle, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed,
            CreateInteractionResponse, CreateInteractionResponseMessage, ReactionType,
        },
        CreateReply, Modal, ReplyHandle,
    },
    std::{future::Future, num::ParseIntError},
};

pub trait LazyPaginationTrait<'a> {
    fn ctx(&self) -> Context<'a>;
}

impl<'a> LazyPaginationTrait<'a> for Context<'a> {
    fn ctx(&self) -> Context<'a> {
        *self
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum PageNumberError {
    #[error("Page must be between {} and {}", min + 1, max + 1)]
    OutOfRange { min: usize, max: usize },
    #[error(transparent)]
    ParseError(#[from] ParseIntError),
}

#[derive(Debug, Modal)]
#[name = "Page Number"]
struct Page {
    #[name = "Page number"]
    #[placeholder = "Page number..."]
    page_number: String,
}

impl Page {
    pub fn page_number(&self, min: usize, max: usize) -> Result<usize, PageNumberError> {
        let page_number = self.page_number.parse::<usize>()?;
        if (min..max).contains(&page_number) {
            Ok(page_number)
        } else {
            Err(PageNumberError::OutOfRange { min, max })
        }
    }
}

pub struct EmbedPaginator<'a> {
    paginator: Paginate<'a, CreateEmbed>,
    ctx: Context<'a>,
    ids: [String; 6],
}

impl<'a> EmbedPaginator<'a> {
    pub fn new(embeds: &'a [CreateEmbed], ctx: Context<'a>) -> Self {
        let ids = [
            format!("{}_fast_rewind", ctx.id()),
            format!("{}_rewind", ctx.id()),
            format!("{}_counter", ctx.id()),
            format!("{}_forward", ctx.id()),
            format!("{}_fast_forward", ctx.id()),
            format!("{}_jump_to", ctx.id()),
        ];
        Self {
            paginator: Paginate::new(embeds),
            ctx,
            ids,
        }
    }

    fn get_paginate_components(&self, disable_all: bool) -> Vec<CreateActionRow> {
        let (left_enabled, right_enabled) = if self.paginator.current_idx() == 0 {
            (false, true)
        } else if self.paginator.current_idx() == self.paginator.max() - 1 {
            (true, false)
        } else {
            (true, true)
        };

        vec![
            CreateActionRow::Buttons(vec![
                CreateButton::new(&self.ids[0])
                    .emoji(ReactionType::Unicode("⏪".to_owned()))
                    .style(ButtonStyle::Success)
                    .disabled(!left_enabled || disable_all),
                CreateButton::new(&self.ids[1])
                    .emoji(ReactionType::Unicode("◀️".to_owned()))
                    .style(ButtonStyle::Secondary)
                    .disabled(!left_enabled || disable_all),
                CreateButton::new(&self.ids[2])
                    .label(format!(
                        "{} / {}",
                        self.paginator.current_idx() + 1,
                        self.paginator.max()
                    ))
                    .disabled(true),
                CreateButton::new(&self.ids[3])
                    .emoji(ReactionType::Unicode("▶️".to_owned()))
                    .style(ButtonStyle::Secondary)
                    .disabled(!right_enabled || disable_all),
                CreateButton::new(&self.ids[4])
                    .emoji(ReactionType::Unicode("⏩".to_owned()))
                    .style(ButtonStyle::Success)
                    .disabled(!right_enabled || disable_all),
            ]),
            CreateActionRow::Buttons(vec![
                CreateButton::new("Jump to page").style(ButtonStyle::Primary)
            ]),
        ]
    }

    async fn send_initial_message(&mut self) -> Result<ReplyHandle<'a>, Error> {
        let embed = self.paginator.first_page().ok_or("Empty paginator")?;
        let components = self.get_paginate_components(false);

        Ok(self
            .ctx
            .send(
                CreateReply::default()
                    .embed(embed.to_owned())
                    .components(components),
            )
            .await?)
    }

    pub async fn start(&mut self, timeout_after: Duration) -> CmdRet {
        let msg = self.send_initial_message().await?;

        let id = self.ctx.id();
        while let Some(press) = ComponentInteractionCollector::new(self.ctx.serenity_context())
            .author_id(self.ctx.author().id)
            .channel_id(self.ctx.channel_id())
            .timeout(timeout_after.to_std()?)
            .filter(move |interaction| interaction.data.custom_id.starts_with(&id.to_string()))
            .await
        {
            let next_embed: Option<&CreateEmbed> = if press.data.custom_id == self.ids[0] {
                self.paginator.first_page()
            } else if press.data.custom_id == self.ids[1] {
                self.paginator.previous_page()
            } else if press.data.custom_id == self.ids[3] {
                self.paginator.next_page()
            } else if press.data.custom_id == self.ids[4] {
                self.paginator.last_page()
            } else {
                panic!("Fatal error: Unmatched ID in embed paginator!")
            };

            if let Some(embed) = next_embed {
                let create_reply = CreateInteractionResponseMessage::new()
                    .components(self.get_paginate_components(false))
                    .embed(embed.clone());

                press
                    .create_response(
                        self.ctx,
                        CreateInteractionResponse::UpdateMessage(create_reply),
                    )
                    .await?;
            }
        }

        msg.edit(
            self.ctx,
            CreateReply::default()
                .embed(
                    (self
                        .paginator
                        .current_page()
                        .ok_or("Pagination pointer left at an invalid position")?)
                    .to_owned(),
                )
                .components(self.get_paginate_components(true)),
        )
        .await?;
        Ok(())
    }
}

pub struct LazyEmbedPaginator<Gen, S> {
    paginator: PaginateLazily<S, Gen>,
    state: S,
    ids: [String; 6],
}

impl<'a, Gen, Fut, S> LazyEmbedPaginator<Gen, S>
where
    Fut: Future<Output = Option<CreateEmbed>>,
    Gen: Fn(S, usize) -> Fut,
    S: Clone + Send + Sync + LazyPaginationTrait<'a>,
{
    pub fn new(generator: Gen, length: usize, state: S) -> Self {
        let ids = [
            format!("{}_fast_rewind", state.ctx().id()),
            format!("{}_rewind", state.ctx().id()),
            format!("{}_counter", state.ctx().id()),
            format!("{}_forward", state.ctx().id()),
            format!("{}_fast_forward", state.ctx().id()),
            format!("{}_jump_to", state.ctx().id()),
        ];
        Self {
            paginator: PaginateLazily::new(length, generator, state.clone()),
            state,
            ids,
        }
    }

    fn get_paginate_components(&self, disable_all: bool) -> Vec<CreateActionRow> {
        let (left_enabled, right_enabled) = match (
            disable_all,
            self.paginator.current_idx(),
            self.paginator.len(),
        ) {
            (true, _, _) => (false, false),
            (false, 0, _) => (false, true),
            (false, idx, len) if idx == len - 1 => (true, false),
            (false, _, _) => (true, true),
        };

        vec![
            CreateActionRow::Buttons(vec![
                CreateButton::new(&self.ids[0])
                    .emoji(ReactionType::Unicode("⏪".to_owned()))
                    .style(ButtonStyle::Success)
                    .disabled(!left_enabled),
                CreateButton::new(&self.ids[1])
                    .emoji(ReactionType::Unicode("◀️".to_owned()))
                    .style(ButtonStyle::Secondary)
                    .disabled(!left_enabled),
                CreateButton::new(&self.ids[2])
                    .label(format!(
                        "{} / {}",
                        self.paginator.current_idx() + 1,
                        self.paginator.len()
                    ))
                    .disabled(true),
                CreateButton::new(&self.ids[3])
                    .emoji(ReactionType::Unicode("▶️".to_owned()))
                    .style(ButtonStyle::Secondary)
                    .disabled(!right_enabled),
                CreateButton::new(&self.ids[4])
                    .emoji(ReactionType::Unicode("⏩".to_owned()))
                    .style(ButtonStyle::Success)
                    .disabled(!right_enabled),
            ]),
            CreateActionRow::Buttons(vec![CreateButton::new(&self.ids[5])
                .style(ButtonStyle::Primary)
                .disabled(!left_enabled && !right_enabled)
                .label("Jump to page")]),
        ]
    }

    async fn send_initial_message(&mut self) -> Result<ReplyHandle<'a>, Error> {
        let embed = self.paginator.first_page().await.ok_or("Empty paginator")?;
        let components = self.get_paginate_components(false);

        Ok(self
            .state
            .ctx()
            .send(
                CreateReply::default()
                    .embed(embed.to_owned())
                    .components(components),
            )
            .await?)
    }

    pub async fn start(&mut self, timeout_after: Duration) -> CmdRet {
        let msg = self.send_initial_message().await?;
        let mut has_responded = false;

        let ctx = self.state.ctx();

        let id = ctx.id();
        while let Some(press) = ComponentInteractionCollector::new(ctx.serenity_context())
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .timeout(timeout_after.to_std()?)
            .filter(move |interaction| interaction.data.custom_id.starts_with(&id.to_string()))
            .await
        {
            let next_embed: Option<CreateEmbed> = match press.data.custom_id.as_str() {
                id if id == self.ids[0] => self.paginator.first_page().await,
                id if id == self.ids[1] => self.paginator.previous_page().await,
                id if id == self.ids[3] => self.paginator.next_page().await,
                id if id == self.ids[4] => self.paginator.last_page().await,

                // "Jump to page click"
                id if id == self.ids[5] => {
                    // spin up task to not block this small "event loop"
                    let handle = tokio::spawn(async move {});
                    let data = match poise::execute_modal_on_component_interaction::<Page>(
                        ctx,
                        press.clone(),
                        None,
                        Some(std::time::Duration::from_secs(10)),
                    )
                    .await?
                    {
                        None => continue,
                        Some(data) => data,
                    };

                    has_responded = true;

                    let validated_num = match data.page_number(0, self.paginator.len()) {
                        Ok(num) => num,
                        Err(err) => {
                            ctx.send(
                                CreateReply::default()
                                    .embed(error_embed(err.to_string()))
                                    .ephemeral(true),
                            )
                            .await?;
                            continue;
                        }
                    };

                    // -1 because the user will enter the number on a 1-based index
                    self.paginator.jump_to(validated_num - 1).await
                }

                _ => unreachable!("Fatal error: Unmatched ID in embed paginator!"),
            };

            if let Some(embed) = next_embed {
                if has_responded {
                    let create_reply = CreateReply::default()
                        .components(self.get_paginate_components(false))
                        .embed(embed.clone());
                    msg.edit(ctx, create_reply).await?;
                } else {
                    let create_reply = CreateInteractionResponseMessage::new()
                        .components(self.get_paginate_components(false))
                        .embed(embed.clone());
                    press
                        .create_response(
                            ctx,
                            CreateInteractionResponse::UpdateMessage(create_reply),
                        )
                        .await?;
                }
            }
        }

        msg.edit(
            ctx,
            CreateReply::default()
                .embed(
                    (self
                        .paginator
                        .current_page()
                        .await
                        .ok_or("Pagination pointer left at an invalid position")?)
                    .to_owned(),
                )
                .components(self.get_paginate_components(true)),
        )
        .await?;
        Ok(())
    }
}
