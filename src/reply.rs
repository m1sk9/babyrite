//! Assembly of the messages Babyrite posts back to the requester.
//!
//! Everything the bot says goes through [`build_messages`], so the reply
//! policy — what is allowed to be mentioned, and what rides in which message —
//! is decided in one place instead of at each call site.

use crate::expand::ExpandedContent;
use crate::utils::defuse_code_fences;
use twilight_model::channel::message::{AllowedMentions, Embed};
use twilight_model::id::{Id, marker::MessageMarker};

/// A message to post, independent of the HTTP request that sends it.
#[derive(Debug)]
pub struct OutgoingMessage {
    /// The message content.
    pub content: Option<String>,
    /// The embeds attached to the message.
    pub embeds: Vec<Embed>,
    /// The message this one replies to.
    pub reply_to: Option<Id<MessageMarker>>,
    /// Who the message may mention.
    pub allowed_mentions: AllowedMentions,
}

/// Builds the messages answering the message `request_id`, in the order they
/// are sent.
///
/// Every message replies to the request, so each expansion stays attached to the
/// link it came from. Embeds ride in a single reply, while each code block gets a
/// reply of its own: a block fills the message content, and a message carries
/// only one.
pub fn build_messages(
    request_id: Id<MessageMarker>,
    results: Vec<ExpandedContent>,
) -> Vec<OutgoingMessage> {
    let mut embeds = Vec::new();
    let mut code_blocks = Vec::new();

    for result in results {
        match result {
            ExpandedContent::Embed(embed) => embeds.push(*embed),
            ExpandedContent::CodeBlock {
                language,
                code,
                metadata,
            } => {
                let code = defuse_code_fences(&code);
                // `allowed_mentions` is set explicitly, and empty: left unset,
                // Discord parses every mention in the content under the bot's
                // permissions. The content is fetched from a repository the
                // requester chose, so that would let it borrow mention rights
                // the requester may not hold. That also leaves `replied_user`
                // off: only the preview pings the requester, so a message with
                // several permalinks does not ping them once per code block.
                code_blocks.push(OutgoingMessage {
                    content: Some(format!("{metadata}\n```{language}\n{code}\n```")),
                    embeds: Vec::new(),
                    reply_to: Some(request_id),
                    allowed_mentions: AllowedMentions::default(),
                });
            }
        }
    }

    if embeds.is_empty() {
        return code_blocks;
    }

    // Only the reply ping is allowed: the quoted message is someone else's
    // content and must not gain mention rights by being echoed by the bot.
    let preview = OutgoingMessage {
        content: None,
        embeds,
        reply_to: Some(request_id),
        allowed_mentions: AllowedMentions {
            replied_user: true,
            ..Default::default()
        },
    };

    std::iter::once(preview).chain(code_blocks).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_util::builder::embed::EmbedBuilder;

    const REQUEST: Id<MessageMarker> = Id::new(1);

    fn embed() -> ExpandedContent {
        ExpandedContent::Embed(Box::new(EmbedBuilder::new().description("quoted").build()))
    }

    fn code_block() -> ExpandedContent {
        ExpandedContent::CodeBlock {
            language: "rust".to_string(),
            code: "fn main() {}".to_string(),
            metadata: "src/main.rs L1".to_string(),
        }
    }

    #[test]
    fn embeds_share_one_message_and_every_code_block_gets_its_own() {
        let messages = build_messages(REQUEST, vec![embed(), code_block(), embed(), code_block()]);

        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].embeds.len(), 2);
    }

    #[test]
    fn embeds_are_sent_before_code_blocks() {
        let messages = build_messages(REQUEST, vec![code_block(), embed()]);

        assert_eq!(messages[0].embeds[0].description.as_deref(), Some("quoted"));
        assert!(messages[1].content.is_some());
    }

    #[test]
    fn nothing_is_sent_without_expanded_content() {
        assert!(build_messages(REQUEST, Vec::new()).is_empty());
    }

    #[test]
    fn code_block_is_fenced_with_its_language_below_its_metadata() {
        let messages = build_messages(REQUEST, vec![code_block()]);

        assert_eq!(
            messages[0].content.as_deref(),
            Some("src/main.rs L1\n```rust\nfn main() {}\n```")
        );
    }

    #[test]
    fn the_preview_replies_to_the_request() {
        let messages = build_messages(REQUEST, vec![embed()]);

        assert_eq!(messages[0].reply_to, Some(REQUEST));
    }

    #[test]
    fn only_the_reply_ping_is_allowed_in_the_preview() {
        let messages = build_messages(REQUEST, vec![embed()]);

        assert_eq!(
            messages[0].allowed_mentions,
            AllowedMentions {
                replied_user: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn a_code_block_mentions_nobody() {
        let messages = build_messages(REQUEST, vec![code_block()]);

        assert_eq!(messages[0].allowed_mentions, AllowedMentions::default());
    }

    #[test]
    fn every_message_replies_to_the_request() {
        let messages = build_messages(REQUEST, vec![embed(), code_block(), code_block()]);

        assert!(messages.iter().all(|m| m.reply_to == Some(REQUEST)));
    }
}
