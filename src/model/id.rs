use serenity::model::prelude::{ChannelId, GuildId, MessageId};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct DiscordIds {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}
