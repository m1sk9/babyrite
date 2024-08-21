use regex::Captures;
use serenity::all::{ChannelId, GuildId, MessageId};

#[derive(Debug, typed_builder::TypedBuilder)]
pub struct BabyriteIDs {
    pub guild: GuildId,
    pub channel: ChannelId,
    pub message: MessageId,
}

impl BabyriteIDs {
    pub fn init(guild_id: u64, channel_id: u64, message_id: u64) -> Self {
        BabyriteIDs::builder()
            .guild(GuildId::new(guild_id))
            .channel(ChannelId::new(channel_id))
            .message(MessageId::new(message_id))
            .build()
    }

    pub fn new(captures: Captures) -> anyhow::Result<Self> {
        let result: Result<Vec<u64>, _> = (1..=3)
            .map(|i| {
                captures
                    .get(i)
                    .ok_or("Failed to get capture group")
                    .and_then(|m| m.as_str().parse::<u64>().map_err(|_| "Failed to parse ID"))
            })
            .collect();

        match result {
            Ok(ids) => Ok(BabyriteIDs::init(ids[0], ids[1], ids[2])),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    pub fn is_own_guild(&self, target_id: GuildId) -> bool {
        target_id == self.guild
    }
}
