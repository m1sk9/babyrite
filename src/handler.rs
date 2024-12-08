use crate::preview::{MessagePreviewIDs, MESSAGE_LINK_REGEX, SKIP_LINK_REGEX};
use serenity::all::{Context, CreateAllowedMentions, Message, PermissionOverwriteType, Ready};
use serenity::builder::CreateMessage;
use serenity::client::EventHandler;
use serenity::gateway::ActivityData;
use serenity::model::Permissions;
use tracing::{debug, info};

pub struct BabyriteHandler;

#[serenity::async_trait]
impl EventHandler for BabyriteHandler {
    async fn message(&self, ctx: Context, message: Message) {
        let config = crate::config::BabyriteConfig::get();

        if message.author.bot {
            return;
        }

        if !MESSAGE_LINK_REGEX.is_match(&message.content)
            || SKIP_LINK_REGEX.is_match(&message.content)
        {
            return;
        }

        let ids = match MessagePreviewIDs::find_from_str(&message.content) {
            Ok(ids) => ids,
            Err(e) => {
                debug!(name: "babyrite Message", "Failed to parse message: {:?}", e);
                return;
            }
        };
        debug!(name: "babyrite Message", "Found ids: {:?}", ids);

        // Ignore if the message is not in the same guild
        if ids.guild_id != message.guild_id.unwrap() {
            return;
        }

        let is_private = message
            .channel(&ctx)
            .await
            .unwrap()
            .guild()
            .unwrap()
            .permission_overwrites
            .iter()
            .any(|p| {
                matches!(p.kind, PermissionOverwriteType::Role(_))
                    && p.deny.contains(Permissions::VIEW_CHANNEL)
            });

        let preview = match ids.get_preview(&ctx, is_private).await {
            Ok(preview) => preview,
            Err(e) => {
                debug!(name: "babyrite Message", "Failed to get preview: {:?}", e);
                return;
            }
        };
        debug!(name: "babyrite Message", "Found preview: {:?}", preview);

        let embed = MessagePreviewIDs::generate_preview(preview);
        let reply = CreateMessage::default()
            .embed(embed)
            .reference_message(&message)
            .allowed_mentions(CreateAllowedMentions::new().replied_user(config.preview.is_mention));
        if let Err(why) = message.channel_id.send_message(&ctx.http, reply).await {
            debug!(name: "babyrite Message", "Failed to send message: {:?}", why);
        }
    }

    async fn ready(&self, ctx: Context, client: Ready) {
        info!(name: "babyrite Initialize Ready", "babyrite is ready! Connected as {}, id: {}", &client.user.name, &client.user.id);
        debug!(name: "babyrite Initialize Ready", "client: {:?}", client);
        ctx.set_activity(Some(ActivityData::playing(format!(
            "v{}",
            env!("CARGO_PKG_VERSION")
        ))));
    }
}
