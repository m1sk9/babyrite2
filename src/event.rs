use crate::{adapters::message::send_citation_embed, model::id::DiscordIds, VERSION};
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::{
    async_trait,
    gateway::ActivityData,
    model::prelude::{ChannelId, GuildId, Message, MessageId, Ready},
    prelude::{Context, EventHandler},
};
use tracing::{error, info, warn};

pub struct EvHandler;

static MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});
// 引用スキップ機能の正規表現
static SKIP_MESSAGE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"<https://(?:ptb\.|canary\.)?discord\.com/channels/\d+/\d+/\d+>").unwrap()
});

#[async_trait]
impl EventHandler for EvHandler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.is_private() || message.author.bot {
            return;
        }

        let content = &message.content;
        if !MESSAGE_LINK_REGEX.is_match(content) || SKIP_MESSAGE_LINK_REGEX.is_match(content) {
            return;
        }
        let matched_str = MESSAGE_LINK_REGEX.find(content).unwrap().as_str();

        info!("引用を開始します...: Request by {}", message.author.name);

        if let Some(triple) = extract_ids_from_link(matched_str) {
            if triple.guild_id == message.guild_id.unwrap() {
                if let Err(why) = send_citation_embed(triple, &ctx.http, &message).await {
                    error!("{:?}", why)
                }
            } else {
                warn!("メッセージ送信元ギルドと一致しないため, 引用をキャンセルしました")
            }
        } else {
            warn!("IDの取り出しに失敗したため, 引用をキャンセルしました")
        }

        info!("引用が完了しました.");
    }

    async fn ready(&self, ctx: Context, bot: Ready) {
        ctx.set_activity(Some(ActivityData::playing(format!(
            "babyrite v{}",
            VERSION
        ))));

        info!(
            "Connected to {name}(ID:{id}). (Using babyrite v{version}).",
            name = bot.user.name,
            id = bot.user.id,
            version = VERSION
        )
    }
}

fn extract_ids_from_link(message_link: &str) -> Option<DiscordIds> {
    let captures = MESSAGE_LINK_REGEX.captures(message_link)?;

    // GuildId
    let first = captures
        .get(1)
        .and_then(|m| m.as_str().parse::<u64>().ok())?;
    // ChannelId
    let second = captures
        .get(2)
        .and_then(|m| m.as_str().parse::<u64>().ok())?;
    // MessageId
    let third = captures
        .get(3)
        .and_then(|m| m.as_str().parse::<u64>().ok())?;

    Some(
        DiscordIds::builder()
            .guild_id(GuildId::new(first))
            .channel_id(ChannelId::new(second))
            .message_id(MessageId::new(third))
            .build(),
    )
}
