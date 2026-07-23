//! Reaction-driven actions on bot-sent previews.
//!
//! A preview posted by the bot carries a reaction (e.g. `🗑️`) that, when
//! pressed by an authorized member, triggers an action on that preview. This
//! module is the mapping from "which emoji" to "which action" plus the
//! authorization rule; `event.rs` wires it to the `reaction_add` gateway
//! event and the Discord API calls needed to carry an action out.
//!
//! Adding a new reaction action means adding a variant to [`ReactionAction`],
//! a case in [`from_emoji`], and a case in [`execute`] — the gateway-side
//! wiring in `event.rs` does not need to change.

use serenity::all::{Context, Message, ReactionType, UserId};

/// A recognized reaction action.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReactionAction {
    /// Delete the bot's preview message.
    DeletePreview,
}

/// The emoji that triggers [`ReactionAction::DeletePreview`].
///
/// `pub(crate)` rather than private: `event.rs` attaches this same emoji to a
/// freshly-sent preview, and reusing the constant keeps the two ends
/// (attaching the reaction, recognizing it) from drifting apart.
pub(crate) const DELETE_PREVIEW_EMOJI: &str = "🗑️";

/// Maps a reaction emoji to a [`ReactionAction`].
///
/// Returns `None` for anything other than the recognized emoji — including
/// custom guild emoji. That `None` is what "every other reaction is ignored"
/// reduces to: an unrecognized emoji simply never reaches an action.
pub fn from_emoji(emoji: &ReactionType) -> Option<ReactionAction> {
    let ReactionType::Unicode(unicode) = emoji else {
        return None;
    };

    // Discord clients are inconsistent about sending the variation selector
    // (U+FE0F) after the base emoji, so it's stripped from both sides before
    // comparing rather than requiring an exact byte match.
    let strip_vs16 = |s: &str| -> String { s.chars().filter(|&c| c != '\u{FE0F}').collect() };

    (strip_vs16(unicode) == strip_vs16(DELETE_PREVIEW_EMOJI))
        .then_some(ReactionAction::DeletePreview)
}

/// Decides whether `reactor` may trigger a reaction action.
///
/// Authorized when the reactor is the original preview requester, or holds
/// `MANAGE_MESSAGES`. When the requester can't be determined (`requester` is
/// `None`), only the permission grants access — this is stricter than
/// treating an unresolved requester as "nobody to exclude", which would let
/// any member act on such a preview.
pub fn is_authorized(
    requester: Option<UserId>,
    reactor: UserId,
    reactor_can_manage_messages: bool,
) -> bool {
    requester == Some(reactor) || reactor_can_manage_messages
}

/// Executes `action` against the preview message that was reacted to.
#[cfg_attr(coverage_nightly, coverage(off))]
pub async fn execute(ctx: &Context, preview: &Message, action: ReactionAction) {
    match action {
        ReactionAction::DeletePreview => {
            if let Err(e) = preview.delete(&ctx.http).await {
                tracing::error!(error = ?e, "failed to delete preview");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::all::EmojiId;

    fn user(id: u64) -> UserId {
        UserId::new(id)
    }

    #[test]
    fn from_emoji_recognizes_the_fully_qualified_trash_emoji() {
        let emoji = ReactionType::Unicode("🗑️".to_string());
        assert_eq!(from_emoji(&emoji), Some(ReactionAction::DeletePreview));
    }

    #[test]
    fn from_emoji_recognizes_the_trash_emoji_without_a_variation_selector() {
        // Some clients send U+1F5D1 alone, without the trailing U+FE0F.
        let emoji = ReactionType::Unicode("\u{1F5D1}".to_string());
        assert_eq!(from_emoji(&emoji), Some(ReactionAction::DeletePreview));
    }

    #[test]
    fn from_emoji_ignores_unrelated_unicode_emoji() {
        let emoji = ReactionType::Unicode("👍".to_string());
        assert_eq!(from_emoji(&emoji), None);
    }

    #[test]
    fn from_emoji_ignores_custom_guild_emoji() {
        let emoji = ReactionType::Custom {
            animated: false,
            id: EmojiId::new(123),
            name: Some("custom".to_string()),
        };
        assert_eq!(from_emoji(&emoji), None);
    }

    #[test]
    fn is_authorized_allows_the_original_requester() {
        assert!(is_authorized(Some(user(1)), user(1), false));
    }

    #[test]
    fn is_authorized_allows_a_manage_messages_holder() {
        assert!(is_authorized(Some(user(1)), user(2), true));
    }

    #[test]
    fn is_authorized_rejects_an_unrelated_reactor_without_permission() {
        assert!(!is_authorized(Some(user(1)), user(2), false));
    }

    #[test]
    fn is_authorized_rejects_an_unknown_requester_without_permission() {
        assert!(!is_authorized(None, user(2), false));
    }

    #[test]
    fn is_authorized_allows_an_unknown_requester_with_permission() {
        assert!(is_authorized(None, user(2), true));
    }
}
