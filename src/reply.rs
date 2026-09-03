//! Assembly of the messages Babyrite posts back to the requester.
//!
//! Everything the bot says goes through [`build_messages`], so the reply
//! policy — what is allowed to be mentioned, and what rides in which message —
//! is decided in one place instead of at each call site.

use crate::expand::ExpandedContent;
use crate::utils::defuse_code_fences;
use serenity::all::{CreateAllowedMentions, CreateMessage, Message};

/// Builds the messages answering `request`, in the order they are sent.
///
/// Embeds ride in a single reply, while each code block gets a message of its
/// own: a block fills the message content, and a message carries only one.
pub fn build_messages(request: &Message, results: Vec<ExpandedContent>) -> Vec<CreateMessage> {
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
                // the requester may not hold.
                code_blocks.push(
                    CreateMessage::new()
                        .content(format!("{metadata}\n```{language}\n{code}\n```"))
                        .allowed_mentions(CreateAllowedMentions::new()),
                );
            }
        }
    }

    if embeds.is_empty() {
        return code_blocks;
    }

    // Only the reply ping is allowed: the quoted message is someone else's
    // content and must not gain mention rights by being echoed by the bot.
    let preview = CreateMessage::new()
        .embeds(embeds)
        .reference_message(request)
        .allowed_mentions(CreateAllowedMentions::new().replied_user(true));

    std::iter::once(preview).chain(code_blocks).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::all::CreateEmbed;

    /// `CreateMessage` has neither getters nor `PartialEq`, so what it would
    /// send is inspected through `Debug` rather than adding a JSON dependency
    /// used by nothing but these tests.
    fn rendered(message: &CreateMessage) -> String {
        format!("{message:?}")
    }

    fn embed() -> ExpandedContent {
        ExpandedContent::Embed(Box::new(CreateEmbed::new().description("quoted")))
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
        let messages = build_messages(
            &Message::default(),
            vec![embed(), code_block(), embed(), code_block()],
        );

        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn embeds_are_sent_before_code_blocks() {
        let messages = build_messages(&Message::default(), vec![code_block(), embed()]);

        assert!(rendered(&messages[0]).contains("quoted"));
        assert!(rendered(&messages[1]).contains("src/main.rs L1"));
    }

    #[test]
    fn nothing_is_sent_without_expanded_content() {
        assert!(build_messages(&Message::default(), Vec::new()).is_empty());
    }

    #[test]
    fn code_block_is_fenced_with_its_language_below_its_metadata() {
        let messages = build_messages(&Message::default(), vec![code_block()]);

        assert!(rendered(&messages[0]).contains(r"src/main.rs L1\n```rust\nfn main() {}\n```"));
    }

    #[test]
    fn only_the_reply_ping_is_allowed_in_the_preview() {
        let messages = build_messages(&Message::default(), vec![embed()]);

        let rendered = rendered(&messages[0]);
        assert!(rendered.contains("parse: [], users: [], roles: [], replied_user: Some(true)"));
    }

    #[test]
    fn a_code_block_mentions_nobody() {
        let messages = build_messages(&Message::default(), vec![code_block()]);

        let rendered = rendered(&messages[0]);
        assert!(rendered.contains("parse: [], users: [], roles: [], replied_user: None"));
    }
}
