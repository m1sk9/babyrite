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

use super::{ExpandContext, ExpandError, ExpandedContent, LinkExpander};
use crate::cache::CacheArgs;
use crate::config::BabyriteConfig;
use serenity::futures::future::join_all;

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
#[derive(Debug)]
pub struct Preview {
    /// The message to preview.
    pub message: Message,
    /// The channel containing the message.
    pub channel: GuildChannel,
}

/// Discord message link expander.
pub struct DiscordExpander;

#[serenity::async_trait]
impl LinkExpander for DiscordExpander {
    /// Discord link expansion is the bot's core function and has no feature flag.
    fn enabled(&self, _config: &BabyriteConfig) -> bool {
        true
    }

    /// Expands Discord message links into embed previews.
    ///
    /// The source channel is resolved once — the expanded preview is posted
    /// there, so it is needed to verify each link target is at least as visible
    /// as that channel. If it cannot be resolved, Discord expansion is skipped
    /// entirely (other expanders are unaffected).
    ///
    /// Whether a link may be expanded at all is decided by [`Preview::get`].
    /// The rejections it reports are expected outcomes and are logged at
    /// `debug`; only genuine failures reach `error` (see
    /// [`PreviewError::is_policy_rejection`]).
    #[cfg_attr(coverage_nightly, coverage(off))]
    async fn expand_all(&self, cx: &ExpandContext<'_>) -> Vec<ExpandedContent> {
        let links = MessageLinkIDs::parse_all(&cx.message.content);
        if links.is_empty() {
            return Vec::new();
        }
        tracing::debug!(count = links.len(), "parsed Discord links");

        let source_channel = match (CacheArgs {
            guild_id: cx.guild_id,
            channel_id: cx.message.channel_id,
        })
        .get(cx.ctx)
        .await
        {
            Ok(channel) => channel,
            Err(e) => {
                tracing::error!(error = %e, "failed to resolve source channel");
                return Vec::new();
            }
        };

        join_all(links.iter().map(|ids| ids.fetch(cx.ctx, &source_channel)))
            .await
            .into_iter()
            .filter_map(|result| match result {
                Ok(content) => Some(content),
                Err(ExpandError::Discord(e)) if e.is_policy_rejection() => {
                    tracing::debug!(error = %e, "skipped Discord link by visibility policy");
                    None
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to expand Discord link");
                    None
                }
            })
            .collect()
    }
}

/// Errors that can occur when generating a Discord message preview.
#[derive(thiserror::Error, Debug)]
pub enum PreviewError {
    /// The link points into a guild other than the one it was posted in.
    #[error("The link points to another guild, which cannot be expanded.")]
    CrossGuild,
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
    // Boxed: `serenity::Error` is 136 bytes and would otherwise dominate the
    // size of every `Result` in this module (`clippy::result_large_err`).
    #[allow(clippy::enum_variant_names)]
    #[error(transparent)]
    SerenityError(#[from] Box<serenity::Error>),
}

impl PreviewError {
    /// Whether this is the visibility policy working as designed rather than a
    /// failure.
    ///
    /// Rejections happen during normal use, so callers log them at `debug` and
    /// keep `error` for cases that need attention.
    pub fn is_policy_rejection(&self) -> bool {
        // Matched exhaustively rather than with a `_` arm so that a new variant
        // has to declare its severity instead of silently counting as a failure.
        match self {
            Self::CrossGuild | Self::Nsfw | Self::Permission => true,
            Self::Cache | Self::SerenityError(_) => false,
        }
    }
}

impl MessageLinkIDs {
    /// Parses all Discord message links from the given text.
    ///
    /// Returns a `Vec<MessageLinkIDs>` containing all valid message links found.
    /// The shared link policy applies (see [`super::parse_links`]): angle-bracket
    /// wrapped and duplicate URLs are ignored, and at most 3 links are returned.
    pub fn parse_all(text: &str) -> Vec<MessageLinkIDs> {
        super::parse_links(text, &MESSAGE_LINK_REGEX, |captures| {
            Some(MessageLinkIDs {
                guild_id: GuildId::new(captures.get(1)?.as_str().parse().ok()?),
                channel_id: ChannelId::new(captures.get(2)?.as_str().parse().ok()?),
                message_id: MessageId::new(captures.get(3)?.as_str().parse().ok()?),
            })
        })
    }

    /// Fetches the linked message and returns an embed preview.
    ///
    /// `source_channel` is the channel where the request originated. It is used to
    /// ensure the linked content is not exposed to members who could not otherwise
    /// view it (see [`Preview::get`]).
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[tracing::instrument(
        skip(self, ctx, source_channel),
        fields(
            guild_id = %self.guild_id,
            channel_id = %self.channel_id,
            message_id = %self.message_id,
        )
    )]
    pub async fn fetch(
        &self,
        ctx: &Context,
        source_channel: &GuildChannel,
    ) -> Result<ExpandedContent, ExpandError> {
        let Preview { message, channel } = Preview::get(self, ctx, source_channel).await?;

        let author_icon_url = message.author.avatar_url().unwrap_or_default();
        let embed = SerenityEmbed::builder()
            .description(message.content)
            .author_name(message.author.name)
            .author_icon_url(author_icon_url)
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

/// Returns `true` for thread channel types.
///
/// Threads do not carry their own permission overwrites; their visibility
/// follows the parent channel. This is used to decide whether visibility must be
/// resolved against the parent (see [`permission_channel`]).
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

/// Returns `true` if any per-member overwrite grants `VIEW_CHANNEL`.
///
/// This is how Discord represents a channel made private by adding individual
/// users. Access granted this way is invisible to [`viewing_roles`], so its
/// presence means the role set understates who can see the channel.
fn has_member_view_allow(overwrites: &[PermissionOverwrite]) -> bool {
    overwrites.iter().any(|ow| {
        matches!(ow.kind, PermissionOverwriteType::Member(_))
            && ow.allow.contains(Permissions::VIEW_CHANNEL)
    })
}

/// Returns `true` when a source channel that grants access through per-member
/// overwrites may still be expanded into the target described by `dest_roles`.
///
/// Members holding such a grant are not represented in the source's role set, so
/// the subset comparison in [`check_visibility`] says nothing about whether they
/// can view the target. The only target they are provably allowed to see is one
/// `@everyone` can view — which, since [`viewing_roles`] treats `@everyone` as an
/// ordinary role, is exactly `dest_roles` containing it.
///
/// Sources without such a grant are fully described by their role set and are
/// left to the subset comparison.
fn member_granted_source_is_safe(
    source_overwrites: &[PermissionOverwrite],
    dest_roles: &HashSet<RoleId>,
    everyone_role_id: RoleId,
) -> bool {
    !has_member_view_allow(source_overwrites) || dest_roles.contains(&everyone_role_id)
}

/// Returns `true` when a link crosses a guild boundary.
///
/// Roles, permission overwrites and the `@everyone` id are all guild-local, so
/// the role-set comparison in [`check_visibility`] cannot judge a channel in
/// another guild — and the bot may not even be a member of that guild. Such
/// links are refused outright rather than judged.
fn is_cross_guild(link: GuildId, source: GuildId) -> bool {
    link != source
}

/// Returns `true` when the link's visibility must be validated against the
/// request's source channel.
///
/// A link pointing back into the same channel the request came from is always
/// safe to expand: the reply lands in that very channel, so it cannot expose
/// anything its readers cannot already see. Such links need no visibility
/// checks, while links to any other channel do.
fn requires_visibility_check(target: ChannelId, source: ChannelId) -> bool {
    target != source
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

    let overwrite_by_role: HashMap<RoleId, (Permissions, Permissions)> = overwrites
        .iter()
        .filter_map(|ow| match ow.kind {
            PermissionOverwriteType::Role(id) => Some((id, (ow.allow, ow.deny))),
            _ => None,
        })
        .collect();

    let mut set = HashSet::new();
    for (&role_id, &perms) in role_perms {
        let base = everyone_base | perms;
        if base.contains(Permissions::ADMINISTRATOR) {
            set.insert(role_id);
            continue;
        }

        let mut allowed = base.contains(Permissions::VIEW_CHANNEL);
        for target in [everyone_role_id, role_id] {
            if let Some(&(allow, deny)) = overwrite_by_role.get(&target) {
                if deny.contains(Permissions::VIEW_CHANNEL) {
                    allowed = false;
                }
                if allow.contains(Permissions::VIEW_CHANNEL) {
                    allowed = true;
                }
            }
        }

        if allowed {
            set.insert(role_id);
        }
    }
    set
}

/// Resolves the channel that carries the properties a thread inherits — its
/// permission overwrites and its NSFW flag.
///
/// Threads hold neither of their own, so for any thread the parent channel is
/// fetched and returned. Non-thread channels are returned unchanged. A thread
/// without a `parent_id` is treated as an error.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn permission_channel(
    channel: &GuildChannel,
    ctx: &Context,
) -> Result<GuildChannel, PreviewError> {
    if !is_thread(channel.kind) {
        return Ok(channel.clone());
    }

    let parent_id = channel.parent_id.ok_or(PreviewError::Permission)?;
    CacheArgs {
        guild_id: channel.guild_id,
        channel_id: parent_id,
    }
    .get(ctx)
    .await
    .map_err(|_| PreviewError::Cache)
}

/// Validates that everyone who can view `source_channel` could also view `channel`.
///
/// The expanded content is posted as a single message that all members of
/// `source_channel` can read, so the linked channel must be at least as visible
/// as the source channel to avoid leaking restricted content.
///
/// Both channels must be in the same guild, which [`Preview::get`] guarantees by
/// refusing cross-guild links: the role data this compares them against is
/// guild-local and would be meaningless otherwise.
///
/// Comparison is by role set, which cannot express per-member grants, so those
/// are handled by separate conservative guards: [`has_member_view_deny`] on the
/// target and [`member_granted_source_is_safe`] on the source.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn check_visibility(
    channel: &GuildChannel,
    source_channel: &GuildChannel,
    ctx: &Context,
) -> Result<(), PreviewError> {
    // Private threads cannot be represented by the role-set comparison
    // (membership is per-user), and DMs are outside the guild context, so
    // both are rejected. Public/news threads fall through and are judged via
    // their parent channel.
    if matches!(
        channel.kind,
        ChannelType::PrivateThread | ChannelType::Private
    ) {
        tracing::debug!(kind = ?channel.kind, "rejected: private channel or thread");
        return Err(PreviewError::Permission);
    }

    // Threads follow their parent channel's permissions, so resolve both the
    // link target and the request source to the channel that actually
    // defines visibility before comparing.
    let (dest_perm, source_perm) = tokio::try_join!(
        permission_channel(channel, ctx),
        permission_channel(source_channel, ctx),
    )?;

    // A per-member deny on the target cannot be represented in the role-set
    // comparison below, so reject conservatively.
    if has_member_view_deny(&dest_perm.permission_overwrites) {
        tracing::debug!("rejected: target has a per-member VIEW_CHANNEL deny");
        return Err(PreviewError::Permission);
    }

    let guild_id = source_channel.guild_id;
    let everyone_role_id = RoleId::new(guild_id.get());
    // Clone the role permission map out of the cache so the non-`Send`
    // `GuildRef` is dropped immediately — holding it across an `await` would
    // make the future `!Send` and fail to compile in the event handler.
    let role_perms: HashMap<RoleId, Permissions> = {
        let guild = ctx.cache.guild(guild_id).ok_or(PreviewError::Permission)?;
        guild
            .roles
            .iter()
            .map(|(&id, role)| (id, role.permissions))
            .collect()
    };

    let dest_roles = viewing_roles(
        &dest_perm.permission_overwrites,
        &role_perms,
        everyone_role_id,
    );
    let source_roles = viewing_roles(
        &source_perm.permission_overwrites,
        &role_perms,
        everyone_role_id,
    );
    if !member_granted_source_is_safe(
        &source_perm.permission_overwrites,
        &dest_roles,
        everyone_role_id,
    ) {
        tracing::debug!(
            "rejected: source grants access per member and the target is not visible to everyone"
        );
        return Err(PreviewError::Permission);
    }

    if !source_roles.is_subset(&dest_roles) {
        tracing::debug!(
            source_roles = source_roles.len(),
            dest_roles = dest_roles.len(),
            "rejected: source channel is more visible than the target"
        );
        return Err(PreviewError::Permission);
    }

    Ok(())
}

impl Preview {
    /// Retrieves a preview for the given message link.
    ///
    /// Every rule deciding whether a link may be expanded lives here. In order,
    /// the link must point into the same guild as `source_channel`, and the
    /// linked channel must not be NSFW, must not be a private thread or DM, and
    /// must be viewable by everyone who can view `source_channel`. The expanded
    /// content is posted as a single message that all members of
    /// `source_channel` can read, so the linked channel must be at least as
    /// visible as the source channel to avoid leaking restricted content. Public
    /// and news threads are judged by their parent channel, which holds both the
    /// permission overwrites and the NSFW flag they inherit.
    ///
    /// The guild boundary is checked before the channel is resolved. Roles and
    /// permission overwrites are guild-local, so a link into another guild
    /// cannot be judged at all, and resolving it would spend rate limit on a
    /// guild the bot may not even be in.
    ///
    /// When the link target is the same channel as `source_channel`, the
    /// visibility checks are skipped entirely: the reply lands in that same
    /// channel, so it cannot expose anything its readers cannot already see.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[tracing::instrument(
        skip(args, ctx, source_channel),
        fields(
            guild_id = %args.guild_id,
            channel_id = %args.channel_id,
            message_id = %args.message_id,
        )
    )]
    async fn get(
        args: &MessageLinkIDs,
        ctx: &Context,
        source_channel: &GuildChannel,
    ) -> Result<Preview, PreviewError> {
        if is_cross_guild(args.guild_id, source_channel.guild_id) {
            tracing::debug!(link_guild_id = %args.guild_id, "rejected: cross-guild link");
            return Err(PreviewError::CrossGuild);
        }

        let caches = CacheArgs {
            // Not `args.guild_id`: that is the value the URL claims. It equals
            // the source guild after the check above, so take it from the
            // resolved channel and keep guild scoping sourced from Discord.
            guild_id: source_channel.guild_id,
            channel_id: args.channel_id,
        };

        let channel = caches.get(ctx).await.map_err(|_| PreviewError::Cache)?;
        tracing::debug!(kind = ?channel.kind, nsfw = channel.nsfw, "resolved target channel");

        // Judged on the parent for threads, not on `channel.nsfw` directly:
        // Discord omits `nsfw` from thread objects because threads inherit it,
        // and serenity defaults the absent field to `false`, so every thread
        // under an NSFW channel would otherwise slip past this gate.
        let age_gate = permission_channel(&channel, ctx).await?;
        if age_gate.nsfw {
            tracing::debug!("rejected: target channel is NSFW");
            return Err(PreviewError::Nsfw);
        }

        // When the link points to the same channel the request came from, the
        // expansion is posted back into that very channel. Every member who can
        // read the reply can already read the original message, so there is
        // nothing to leak and the visibility checks can be skipped. This
        // notably covers quoting within a private channel, which would otherwise
        // be rejected by the per-member deny guard.
        if requires_visibility_check(args.channel_id, source_channel.id) {
            check_visibility(&channel, source_channel, ctx).await?;
        }

        let started = std::time::Instant::now();
        let message = channel
            .message(&ctx.http, args.message_id)
            .await
            .map_err(Box::new)?;
        tracing::debug!(
            elapsed_ms = started.elapsed().as_millis(),
            "fetched linked message"
        );
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

    /// Builds a per-member overwrite allowing and/or denying VIEW_CHANNEL.
    fn member_ow(id: u64, allow_view: bool, deny_view: bool) -> PermissionOverwrite {
        let bit = |set: bool| {
            if set {
                Permissions::VIEW_CHANNEL
            } else {
                Permissions::empty()
            }
        };
        PermissionOverwrite {
            allow: bit(allow_view),
            deny: bit(deny_view),
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
        assert!(has_member_view_deny(&[member_ow(5, false, true)]));
        // role deny is not a member deny
        assert!(!has_member_view_deny(&[role_ow(1, false, true)]));
        // member allow (not deny) does not trigger
        assert!(!has_member_view_deny(&[member_ow(5, true, false)]));
        assert!(!has_member_view_deny(&[]));
    }

    #[test]
    fn member_view_allow_detected() {
        assert!(has_member_view_allow(&[member_ow(5, true, false)]));
        // A role allow is not a per-member grant.
        assert!(!has_member_view_allow(&[role_ow(1, true, false)]));
        // A member deny is not a grant.
        assert!(!has_member_view_allow(&[member_ow(5, false, true)]));
        assert!(!has_member_view_allow(&[]));
    }

    #[test]
    fn member_granted_source_may_only_expand_public_targets() {
        let public_target = HashSet::from([EVERYONE, MEMBER]);
        let restricted_target = HashSet::from([MEMBER]);
        let granted = [member_ow(5, true, false)];

        // Whoever was added individually can read anything `@everyone` can.
        assert!(member_granted_source_is_safe(
            &granted,
            &public_target,
            EVERYONE
        ));
        // Their access to a restricted target cannot be established from roles.
        assert!(!member_granted_source_is_safe(
            &granted,
            &restricted_target,
            EVERYONE
        ));
    }

    #[test]
    fn role_only_source_is_left_to_the_subset_check() {
        let restricted_target = HashSet::from([MEMBER]);
        // A source described entirely by roles imposes no extra restriction here,
        // whatever the target looks like.
        assert!(member_granted_source_is_safe(
            &[],
            &restricted_target,
            EVERYONE
        ));
        assert!(member_granted_source_is_safe(
            &[role_ow(MEMBER.get(), true, false)],
            &restricted_target,
            EVERYONE
        ));
        // A member deny on the source narrows it, which is already conservative.
        assert!(member_granted_source_is_safe(
            &[member_ow(5, false, true)],
            &restricted_target,
            EVERYONE
        ));
    }

    #[test]
    fn same_channel_skips_visibility_check() {
        let chan = ChannelId::new(42);
        // Quoting within the same channel needs no visibility check.
        assert!(!requires_visibility_check(chan, chan));
        // A link to a different channel still requires validation.
        assert!(requires_visibility_check(chan, ChannelId::new(99)));
    }

    #[test]
    fn cross_guild_links_are_rejected() {
        let guild = GuildId::new(7);
        // A link into the guild it was posted in may be judged further.
        assert!(!is_cross_guild(guild, guild));
        // A link from any other guild is refused outright.
        assert!(is_cross_guild(GuildId::new(8), guild));
    }

    #[test]
    fn only_visibility_rejections_count_as_policy() {
        // Expected outcomes of the visibility policy: logged at debug.
        assert!(PreviewError::CrossGuild.is_policy_rejection());
        assert!(PreviewError::Nsfw.is_policy_rejection());
        assert!(PreviewError::Permission.is_policy_rejection());
        // Genuine failures: logged at error.
        assert!(!PreviewError::Cache.is_policy_rejection());
        assert!(
            !PreviewError::SerenityError(Box::new(serenity::Error::Other("boom")))
                .is_policy_rejection()
        );
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

    #[test]
    fn missing_everyone_role_defaults_to_no_base() {
        // @everyone absent from the role map -> base falls back to empty,
        // so a role with no view permission and no overwrite cannot view.
        let mut roles = HashMap::new();
        roles.insert(MEMBER, Permissions::empty());

        let viewing = viewing_roles(&[], &roles, EVERYONE);
        assert!(!viewing.contains(&MEMBER));

        // The same role gains access once an overwrite allows it.
        let ow = [role_ow(MEMBER.get(), true, false)];
        let viewing = viewing_roles(&ow, &roles, EVERYONE);
        assert!(viewing.contains(&MEMBER));
    }
}
