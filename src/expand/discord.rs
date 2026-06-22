//! Discord message link expansion.
//!
//! This module provides functionality for parsing Discord message links
//! and generating embed previews of the linked messages.
//!
//! Migrated from `preview.rs` with support for multiple link expansion.

use regex::Regex;
use serenity::all::{
    ChannelId, ChannelType, Context, GuildChannel, GuildId, Message, MessageId,
    PermissionOverwrite, PermissionOverwriteType, Permissions, RoleId,
};
use serenity_builder::model::embed::SerenityEmbed;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use url::Url;

use super::{ExpandError, ExpandedContent};
use crate::cache::CacheArgs;

/// Regex pattern for matching Discord message links.
///
/// Supports production, PTB, and Canary Discord URLs.
pub static MESSAGE_LINK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://(?:ptb\.|canary\.)?discord\.com/channels/(\d+)/(\d+)/(\d+)").unwrap()
});

/// Parsed IDs from a Discord message link.
#[derive(Debug)]
pub struct MessageLinkIDs {
    /// The guild ID from the message link.
    pub guild_id: GuildId,
    /// The channel ID from the message link.
    pub channel_id: ChannelId,
    /// The message ID from the message link.
    pub message_id: MessageId,
}

/// A preview containing the message and its channel.
#[derive(serde::Deserialize, Debug)]
pub struct Preview {
    /// The message to preview.
    pub message: Message,
    /// The channel containing the message.
    pub channel: GuildChannel,
}

/// Errors that can occur when generating a Discord message preview.
#[derive(thiserror::Error, Debug)]
pub enum PreviewError {
    /// Failed to retrieve channel information from cache.
    #[error("Failed to retrieve from cache.")]
    Cache,
    /// The target channel is marked as NSFW.
    #[error("NSFW content previews are not permitted, but the channel is marked as NSFW.")]
    Nsfw,
    /// The target channel is private or a private thread.
    #[error("The channel is a private channel or private thread.")]
    Permission,
    /// An error occurred while communicating with Discord.
    #[allow(clippy::enum_variant_names)]
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
}

impl MessageLinkIDs {
    /// Parses all Discord message links from the given text.
    ///
    /// Returns a `Vec<MessageLinkIDs>` containing all valid message links found.
    ///
    /// Note: Duplicate URLs are ignored, and a maximum of 3 links are returned.
    pub fn parse_all(text: &str) -> Vec<MessageLinkIDs> {
        let mut seen_urls = HashSet::new();
        MESSAGE_LINK_REGEX
            .captures_iter(text)
            .filter_map(|captures| {
                let m = captures.get(0)?;
                let full_url = m.as_str();
                // Skip URLs wrapped in angle brackets (e.g., <https://...>)
                if m.start() > 0 && text.as_bytes()[m.start() - 1] == b'<' {
                    return None;
                }
                if !seen_urls.insert(full_url.to_string()) {
                    return None;
                }

                let url = Url::parse(full_url).ok()?;

                if !matches!(
                    url.domain(),
                    Some("discord.com") | Some("canary.discord.com") | Some("ptb.discord.com")
                ) {
                    return None;
                }

                let guild_id = GuildId::new(captures.get(1)?.as_str().parse().ok()?);
                let channel_id = ChannelId::new(captures.get(2)?.as_str().parse().ok()?);
                let message_id = MessageId::new(captures.get(3)?.as_str().parse().ok()?);

                Some(MessageLinkIDs {
                    guild_id,
                    channel_id,
                    message_id,
                })
            })
            .take(3) // Limit to maximum 3 links
            .collect()
    }

    /// Fetches the linked message and returns an embed preview.
    ///
    /// `source_channel` is the channel where the request originated. It is used to
    /// ensure the linked content is not exposed to members who could not otherwise
    /// view it (see [`Preview::get`]).
    pub async fn fetch(
        &self,
        ctx: &Context,
        source_channel: &GuildChannel,
    ) -> Result<ExpandedContent, ExpandError> {
        let preview = Preview::get(self, ctx, source_channel).await?;
        let (message, channel) = (preview.message, preview.channel);

        let embed = SerenityEmbed::builder()
            .description(message.content)
            .author_name(message.author.name.clone())
            .author_icon_url(message.author.avatar_url().unwrap_or_default())
            .footer_text(channel.name)
            .timestamp(message.timestamp)
            .color(0x7A4AFFu32)
            .image_url(
                message
                    .attachments
                    .first()
                    .map(|a| a.url.clone())
                    .unwrap_or_default(),
            )
            .build();

        Ok(ExpandedContent::Embed(Box::new(embed)))
    }
}

/// Returns `true` for thread channel types, which are always rejected.
///
/// Threads inherit permissions from their parent channel, which this module does
/// not resolve, so they are never expanded.
fn is_thread(kind: ChannelType) -> bool {
    matches!(
        kind,
        ChannelType::NewsThread | ChannelType::PublicThread | ChannelType::PrivateThread
    )
}

/// Returns `true` if any per-member overwrite denies `VIEW_CHANNEL`.
///
/// Per-member overwrites cannot be captured by the role-set comparison in
/// [`viewing_roles`], so their presence forces a conservative rejection.
fn has_member_view_deny(overwrites: &[PermissionOverwrite]) -> bool {
    overwrites.iter().any(|ow| {
        matches!(ow.kind, PermissionOverwriteType::Member(_))
            && ow.deny.contains(Permissions::VIEW_CHANNEL)
    })
}

/// Computes the set of roles that can effectively `VIEW_CHANNEL` a channel.
///
/// `@everyone` (role id == guild id) is treated as a normal role and included in
/// the result when applicable. For each role the effective permission is
/// `@everyone perms | role perms`; a role with `ADMINISTRATOR` always views the
/// channel. Otherwise the channel's `@everyone` overwrite is applied first, then
/// the role's own overwrite, each as deny-then-allow.
fn viewing_roles(
    overwrites: &[PermissionOverwrite],
    role_perms: &HashMap<RoleId, Permissions>,
    everyone_role_id: RoleId,
) -> HashSet<RoleId> {
    let everyone_base = role_perms
        .get(&everyone_role_id)
        .copied()
        .unwrap_or_else(Permissions::empty);

    let mut set = HashSet::new();
    for (&role_id, &perms) in role_perms {
        let base = everyone_base | perms;
        if base.contains(Permissions::ADMINISTRATOR) {
            set.insert(role_id);
            continue;
        }

        let mut allowed = base.contains(Permissions::VIEW_CHANNEL);
        for target in [everyone_role_id, role_id] {
            for ow in overwrites {
                if matches!(ow.kind, PermissionOverwriteType::Role(id) if id == target) {
                    if ow.deny.contains(Permissions::VIEW_CHANNEL) {
                        allowed = false;
                    }
                    if ow.allow.contains(Permissions::VIEW_CHANNEL) {
                        allowed = true;
                    }
                }
            }
        }

        if allowed {
            set.insert(role_id);
        }
    }
    set
}

impl Preview {
    /// Retrieves a preview for the given message link.
    ///
    /// Validates that the linked channel is not NSFW, is not a thread, and that
    /// everyone who can view the request's `source_channel` could also view the
    /// linked channel. The expanded content is posted as a single message that all
    /// members of `source_channel` can read, so the linked channel must be at least
    /// as visible as the source channel to avoid leaking restricted content.
    async fn get(
        args: &MessageLinkIDs,
        ctx: &Context,
        source_channel: &GuildChannel,
    ) -> Result<Preview, PreviewError> {
        let caches = CacheArgs {
            guild_id: args.guild_id,
            channel_id: args.channel_id,
        };

        let channel = caches.get(ctx).await.map_err(|_| PreviewError::Cache)?;

        if channel.nsfw {
            return Err(PreviewError::Nsfw);
        }

        if is_thread(channel.kind) || matches!(channel.kind, ChannelType::Private) {
            return Err(PreviewError::Permission);
        }

        // A per-member deny on the linked channel cannot be represented in the
        // role-set comparison below, so reject conservatively.
        if has_member_view_deny(&channel.permission_overwrites) {
            return Err(PreviewError::Permission);
        }

        let everyone_role_id = RoleId::new(args.guild_id.get());
        // Clone the role permission map out of the cache so the `GuildRef` is
        // dropped before the `await` below (holding it across `await` would make
        // the future `!Send` and fail to compile in the event handler).
        let role_perms: HashMap<RoleId, Permissions> = {
            let guild = ctx
                .cache
                .guild(args.guild_id)
                .ok_or(PreviewError::Permission)?;
            guild
                .roles
                .iter()
                .map(|(&id, role)| (id, role.permissions))
                .collect()
        };

        let dest_roles = viewing_roles(
            &channel.permission_overwrites,
            &role_perms,
            everyone_role_id,
        );
        let source_roles = viewing_roles(
            &source_channel.permission_overwrites,
            &role_perms,
            everyone_role_id,
        );
        if !source_roles.is_subset(&dest_roles) {
            return Err(PreviewError::Permission);
        }

        let message = channel.message(&ctx.http, args.message_id).await?;
        Ok(Preview { message, channel })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_link() {
        let text = "https://discord.com/channels/123456789/987654321/111111111";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].guild_id, GuildId::new(123456789));
        assert_eq!(results[0].channel_id, ChannelId::new(987654321));
        assert_eq!(results[0].message_id, MessageId::new(111111111));
    }

    #[test]
    fn parse_ptb_link() {
        let text = "https://ptb.discord.com/channels/123/456/789";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].guild_id, GuildId::new(123));
    }

    #[test]
    fn parse_canary_link() {
        let text = "https://canary.discord.com/channels/123/456/789";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].guild_id, GuildId::new(123));
    }

    #[test]
    fn parse_multiple_links() {
        let text = "https://discord.com/channels/1/2/3 and https://discord.com/channels/4/5/6";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].guild_id, GuildId::new(1));
        assert_eq!(results[1].guild_id, GuildId::new(4));
    }

    #[test]
    fn parse_deduplicates() {
        let text = "https://discord.com/channels/1/2/3 https://discord.com/channels/1/2/3";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn parse_limits_to_three() {
        let text = "\
            https://discord.com/channels/1/2/3 \
            https://discord.com/channels/4/5/6 \
            https://discord.com/channels/7/8/9 \
            https://discord.com/channels/10/11/12";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn parse_no_match() {
        let text = "Just some regular text";
        let results = MessageLinkIDs::parse_all(text);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_ignores_invalid_url() {
        // Non-discord domain should not match (regex anchors to discord.com)
        let text = "https://notdiscord.com/channels/1/2/3";
        let results = MessageLinkIDs::parse_all(text);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_ignores_angle_bracket_link() {
        let text = "<https://discord.com/channels/123/456/789>";
        let results = MessageLinkIDs::parse_all(text);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_mixed_with_text() {
        let text = "Hey check this out https://discord.com/channels/1/2/3 pretty cool right?";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message_id, MessageId::new(3));
    }

    // --- Privacy / permission resolution ---

    use serenity::all::UserId;

    /// Builds a role VIEW_CHANNEL overwrite.
    fn role_ow(id: u64, allow_view: bool, deny_view: bool) -> PermissionOverwrite {
        PermissionOverwrite {
            allow: if allow_view {
                Permissions::VIEW_CHANNEL
            } else {
                Permissions::empty()
            },
            deny: if deny_view {
                Permissions::VIEW_CHANNEL
            } else {
                Permissions::empty()
            },
            kind: PermissionOverwriteType::Role(RoleId::new(id)),
        }
    }

    /// Builds a per-member overwrite that denies VIEW_CHANNEL when `deny_view`.
    fn member_ow(id: u64, deny_view: bool) -> PermissionOverwrite {
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: if deny_view {
                Permissions::VIEW_CHANNEL
            } else {
                Permissions::empty()
            },
            kind: PermissionOverwriteType::Member(UserId::new(id)),
        }
    }

    const EVERYONE: RoleId = RoleId::new(1);
    const MEMBER: RoleId = RoleId::new(100);
    const SPECIAL: RoleId = RoleId::new(200);
    const ADMIN: RoleId = RoleId::new(300);

    #[test]
    fn thread_kinds_detected() {
        assert!(is_thread(ChannelType::PublicThread));
        assert!(is_thread(ChannelType::NewsThread));
        assert!(is_thread(ChannelType::PrivateThread));
        assert!(!is_thread(ChannelType::Text));
        assert!(!is_thread(ChannelType::Voice));
    }

    #[test]
    fn member_view_deny_detected() {
        assert!(has_member_view_deny(&[member_ow(5, true)]));
        // role deny is not a member deny
        assert!(!has_member_view_deny(&[role_ow(1, false, true)]));
        // member allow (not deny) does not trigger
        assert!(!has_member_view_deny(&[member_ow(5, false)]));
        assert!(!has_member_view_deny(&[]));
    }

    #[test]
    fn public_channels_are_subsets() {
        // @everyone has VIEW_CHANNEL by base permission, no overwrites.
        let mut roles = HashMap::new();
        roles.insert(EVERYONE, Permissions::VIEW_CHANNEL);
        roles.insert(MEMBER, Permissions::empty());

        let viewing = viewing_roles(&[], &roles, EVERYONE);
        assert!(viewing.contains(&EVERYONE));
        assert!(viewing.contains(&MEMBER));
        // source == dest -> subset holds
        assert!(viewing.is_subset(&viewing));
    }

    #[test]
    fn role_gate_allows_matching_member_role() {
        // @everyone has no base view; member role is granted via overwrite.
        let mut roles = HashMap::new();
        roles.insert(EVERYONE, Permissions::empty());
        roles.insert(MEMBER, Permissions::empty());

        let ow = [
            role_ow(EVERYONE.get(), false, true),
            role_ow(MEMBER.get(), true, false),
        ];
        let viewing = viewing_roles(&ow, &roles, EVERYONE);

        assert!(!viewing.contains(&EVERYONE));
        assert!(viewing.contains(&MEMBER));

        // Both source and dest gated identically -> subset holds (expansion allowed).
        let source = viewing_roles(&ow, &roles, EVERYONE);
        assert!(source.is_subset(&viewing));
    }

    #[test]
    fn narrower_target_is_rejected() {
        let mut roles = HashMap::new();
        roles.insert(EVERYONE, Permissions::empty());
        roles.insert(MEMBER, Permissions::empty());
        roles.insert(SPECIAL, Permissions::empty());

        // Source: role-gated, visible to MEMBER.
        let source_ow = [
            role_ow(EVERYONE.get(), false, true),
            role_ow(MEMBER.get(), true, false),
        ];
        let source = viewing_roles(&source_ow, &roles, EVERYONE);

        // Dest: visible only to SPECIAL.
        let dest_ow = [
            role_ow(EVERYONE.get(), false, true),
            role_ow(SPECIAL.get(), true, false),
        ];
        let dest = viewing_roles(&dest_ow, &roles, EVERYONE);

        assert!(source.contains(&MEMBER));
        assert!(!dest.contains(&MEMBER));
        // MEMBER can see source but not dest -> leak -> not a subset.
        assert!(!source.is_subset(&dest));
    }

    #[test]
    fn administrator_always_views() {
        let mut roles = HashMap::new();
        roles.insert(EVERYONE, Permissions::empty());
        roles.insert(ADMIN, Permissions::ADMINISTRATOR);

        // Even with @everyone denied, an ADMINISTRATOR role still views.
        let ow = [role_ow(EVERYONE.get(), false, true)];
        let viewing = viewing_roles(&ow, &roles, EVERYONE);
        assert!(viewing.contains(&ADMIN));
        assert!(!viewing.contains(&EVERYONE));
    }

    #[test]
    fn other_role_overwrite_does_not_affect_role() {
        let mut roles = HashMap::new();
        roles.insert(EVERYONE, Permissions::VIEW_CHANNEL);
        roles.insert(MEMBER, Permissions::empty());
        roles.insert(SPECIAL, Permissions::empty());

        // Only SPECIAL is denied; MEMBER should be unaffected.
        let ow = [role_ow(SPECIAL.get(), false, true)];
        let viewing = viewing_roles(&ow, &roles, EVERYONE);
        assert!(viewing.contains(&MEMBER));
        assert!(!viewing.contains(&SPECIAL));
    }
}
