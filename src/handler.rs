use crate::model::cache::get_channel_list_cache;
use crate::model::config::BABYRITE_CONFIG;
use crate::model::ids::BabyriteIDs;
use crate::model::message::{
    CitationMessage, CitationMessageAuthor, MESSAGE_LINK_REGEX, SKIP_MESSAGE_LINK_REGEX,
};
use serenity::all::{Context, CreateAllowedMentions, CreateMessage, Message, Ready};
use serenity::client::EventHandler;
use serenity::gateway::ActivityData;

pub struct BabyriteHandler;

#[serenity::async_trait]
impl EventHandler for BabyriteHandler {
    async fn message(&self, ctx: Context, message: Message) {
        let config = BABYRITE_CONFIG.get().unwrap();
        if message.author.bot {
            return;
        }

        let content = &message.content;
        if !MESSAGE_LINK_REGEX.is_match(content) || SKIP_MESSAGE_LINK_REGEX.is_match(content) {
            return;
        }
        let captures = MESSAGE_LINK_REGEX
            .captures(MESSAGE_LINK_REGEX.find(content).unwrap().as_str())
            .unwrap();
        let ids = BabyriteIDs::new(captures).unwrap();

        if !config.bypass_guilds && message.guild_id != Some(ids.guild) {
            return;
        }

        let target_channel = match get_channel_list_cache(&ctx, ids.guild, ids.channel).await {
            Ok(channel) => channel,
            Err(e) => {
                tracing::error!(
                    "Failed to retrieve channel. It does not exist or the cache is invalid: {:?}",
                    e
                );
                return;
            }
        };
        let target_message = match target_channel.message(&ctx.http, ids.message).await {
            Ok(message) => message,
            Err(e) => {
                tracing::error!("Failed to retrieve message: {:?}", e);
                return;
            }
        };

        if target_channel.nsfw {
            return;
        }

        if !target_message.embeds.is_empty() && target_message.content.is_empty() {
            return;
        }

        let source_message = CitationMessage::builder()
            .content(target_message.content.clone())
            .author(
                CitationMessageAuthor::builder()
                    .name(target_message.author.name.clone())
                    .icon_url(Some(target_message.author.avatar_url().unwrap_or(
                        "https://cdn.discordapp.com/embed/avatars/0.png".to_string(),
                    )))
                    .build(),
            )
            .channel_name(target_channel.name.clone())
            .create_at(target_message.timestamp)
            .attachment_image_url(target_message.attachments.first().map(|a| a.url.clone()))
            .build();

        let embed = source_message.to_embed();

        tracing::debug!(
            "Target: {:?}({:?}), Embed: {:?}",
            &target_message,
            &target_channel,
            &embed
        );
        if let Err(why) = message
            .channel_id
            .send_message(&ctx.http, {
                CreateMessage::default()
                    .embed(embed)
                    .allowed_mentions(
                        CreateAllowedMentions::default().replied_user(config.citation_mention),
                    )
                    .reference_message(&message.clone())
            })
            .await
        {
            tracing::error!("Failed to send message: {:?}", why);
        }
    }

    async fn ready(&self, ctx: Context, client: Ready) {
        tracing::info!(
            "Connected as {}, id: {}",
            &client.user.name,
            &client.user.id
        );
        ctx.set_activity(Some(ActivityData::playing(format!(
            "babyrite v{}",
            env!("CARGO_PKG_VERSION")
        ))));
        tracing::debug!("client: {:?}", client);
        tracing::info!("babyrite is ready!");
    }
}
