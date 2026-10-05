//! Discord message link expansion.
//!
//! This module provides functionality for parsing Discord message links
//! and generating embed previews of the linked messages.
//!
//! Migrated from `preview.rs` with support for multiple link expansion.

use futures_util::future::{join_all, try_join_all};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use twilight_http::error::ErrorType;
use twilight_model::channel::message::Embed;
use twilight_model::channel::permission_overwrite::{PermissionOverwrite, PermissionOverwriteType};
use twilight_model::channel::{Channel, ChannelType, Message};
use twilight_model::guild::Permissions;
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker},
};
use twilight_model::user::User;
use twilight_util::builder::embed::{
    EmbedAuthorBuilder, EmbedBuilder, EmbedFooterBuilder, ImageSource,
};

use super::{ExpandContext, ExpandError, ExpandedContent, LinkExpander};
use crate::cache::{CacheArgs, RolePermissions};
use crate::config::BabyriteConfig;
use crate::context::BotContext;

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
    pub guild_id: Id<GuildMarker>,
    /// The channel ID from the message link.
    pub channel_id: Id<ChannelMarker>,
    /// The message ID from the message link.
    pub message_id: Id<MessageMarker>,
}

/// A preview containing the message and its channel.
#[derive(Debug)]
pub struct Preview {
    /// The message to preview.
    pub message: Message,
    /// The channel containing the message.
    pub channel: Channel,
}

/// Discord message link expander.
pub struct DiscordExpander;

#[async_trait::async_trait]
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
        .get(&cx.ctx.http)
        .await
        {
            Ok(channel) => channel,
            Err(e) => {
                tracing::error!(error = %e, "failed to resolve source channel");
                return Vec::new();
            }
        };

        join_all(
            links
                .iter()
                .map(|ids| ids.fetch(cx.ctx, cx.guild_id, &source_channel)),
        )
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
    // A trait object rather than twilight's error types: their constructors are
    // private, so a concrete variant could not be built in tests.
    #[error("Failed to fetch the linked message: {0}")]
    Discord(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl From<twilight_http::Error> for PreviewError {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(e: twilight_http::Error) -> Self {
        Self::Discord(Box::new(e))
    }
}

impl From<twilight_http::response::DeserializeBodyError> for PreviewError {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn from(e: twilight_http::response::DeserializeBodyError) -> Self {
        Self::Discord(Box::new(e))
    }
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
            Self::Cache | Self::Discord(_) => false,
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
                guild_id: Id::new_checked(captures.get(1)?.as_str().parse().ok()?)?,
                channel_id: Id::new_checked(captures.get(2)?.as_str().parse().ok()?)?,
                message_id: Id::new_checked(captures.get(3)?.as_str().parse().ok()?)?,
            })
        })
    }

    /// Fetches the linked message and returns an embed preview.
    ///
    /// `source_channel` is the channel where the request originated. It is used to
    /// ensure the linked content is not exposed to members who could not otherwise
    /// view it (see [`Preview::get`]). `guild_id` is the guild the request was
    /// posted in.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[tracing::instrument(
        skip(self, ctx, guild_id, source_channel),
        fields(
            guild_id = %self.guild_id,
            channel_id = %self.channel_id,
            message_id = %self.message_id,
        )
    )]
    pub async fn fetch(
        &self,
        ctx: &BotContext,
        guild_id: Id<GuildMarker>,
        source_channel: &Channel,
    ) -> Result<ExpandedContent, ExpandError> {
        let Preview { message, channel } =
            Preview::get(self, ctx, guild_id, source_channel).await?;

        Ok(ExpandedContent::Embed(Box::new(preview_embed(
            &message, &channel,
        ))))
    }
}

/// Accent colour of a message preview embed.
const PREVIEW_EMBED_COLOUR: u32 = 0x7A4AFF;

/// Renders a fetched message as the embed posted in the requester's channel.
///
/// Optional parts of the source message stay unset rather than being sent as an
/// empty string: Discord rejects `""` where it expects a URL or footer text.
fn preview_embed(message: &Message, channel: &Channel) -> Embed {
    let mut author = EmbedAuthorBuilder::new(message.author.name.clone());
    if let Some(url) = avatar_url(&message.author) {
        author = author.icon_url(ImageSource::url(url).expect("CDN URL is https"));
    }

    let mut embed = EmbedBuilder::new()
        .description(message.content.clone())
        .author(author)
        .timestamp(message.timestamp)
        .color(PREVIEW_EMBED_COLOUR);

    if let Some(name) = &channel.name {
        embed = embed.footer(EmbedFooterBuilder::new(name.clone()));
    }

    if let Some(source) = message
        .attachments
        .first()
        .and_then(|attachment| ImageSource::url(attachment.url.clone()).ok())
    {
        embed = embed.image(source);
    }

    embed.build()
}

/// Returns the CDN URL of `user`'s avatar, or `None` when they have none.
fn avatar_url(user: &User) -> Option<String> {
    let hash = user.avatar?;
    let ext = if hash.is_animated() { "gif" } else { "webp" };
    Some(format!(
        "https://cdn.discordapp.com/avatars/{}/{hash}.{ext}?size=1024",
        user.id
    ))
}

/// Returns `true` for thread channel types.
///
/// Threads do not carry their own permission overwrites; their visibility
/// follows the parent channel. This is used to decide whether visibility must be
/// resolved against the parent (see [`permission_channel`]).
fn is_thread(kind: ChannelType) -> bool {
    matches!(
        kind,
        ChannelType::AnnouncementThread | ChannelType::PublicThread | ChannelType::PrivateThread
    )
}

/// Returns `true` if any overwrite targets a kind Discord added after this
/// library was written.
///
/// Its effect cannot be evaluated by [`Visibility`], so its presence forces a
/// conservative rejection.
fn has_unknown_overwrite(overwrites: &[PermissionOverwrite]) -> bool {
    overwrites
        .iter()
        .any(|ow| matches!(ow.kind, PermissionOverwriteType::Unknown(_)))
}

/// The permission overwrites of `channel`; Discord omits the field when empty.
fn overwrites(channel: &Channel) -> &[PermissionOverwrite] {
    channel.permission_overwrites.as_deref().unwrap_or(&[])
}

/// Returns `true` when a link crosses a guild boundary.
///
/// Roles, permission overwrites and the `@everyone` id are all guild-local, so
/// [`check_visibility`] cannot evaluate a channel in
/// another guild — and the bot may not even be a member of that guild. Such
/// links are refused outright rather than judged.
fn is_cross_guild(link: Id<GuildMarker>, source: Id<GuildMarker>) -> bool {
    link != source
}

/// Returns `true` when the link's visibility must be validated against the
/// request's source channel.
///
/// A link pointing back into the same channel the request came from is always
/// safe to expand: the reply lands in that very channel, so it cannot expose
/// anything its readers cannot already see. Such links need no visibility
/// checks, while links to any other channel do.
fn requires_visibility_check(target: Id<ChannelMarker>, source: Id<ChannelMarker>) -> bool {
    target != source
}

/// Permissions a member needs to read the linked message in a text channel.
///
/// `READ_MESSAGE_HISTORY` is required on top of `VIEW_CHANNEL`: without it a
/// member only sees messages sent while they are watching, never an older one a
/// link points at.
const READ_TARGET: Permissions = Permissions::VIEW_CHANNEL.union(Permissions::READ_MESSAGE_HISTORY);

/// Permissions a member needs to read the expanded reply in the source channel.
const READ_SOURCE: Permissions = Permissions::VIEW_CHANNEL;

/// Permissions a member needs to read the linked message in a channel of `kind`.
///
/// The text chat of voice and stage channels is closed to members who cannot
/// `CONNECT`, even when they can view the channel itself.
fn read_target_requirement(kind: ChannelType) -> Permissions {
    match kind {
        ChannelType::GuildVoice | ChannelType::GuildStageVoice => {
            READ_TARGET | Permissions::CONNECT
        }
        _ => READ_TARGET,
    }
}

/// The `(allow, deny)` pair of a permission overwrite.
type OverwritePair = (Permissions, Permissions);

const NO_OVERWRITE: OverwritePair = (Permissions::empty(), Permissions::empty());

/// A channel's permission overwrites, split by what they target and masked to
/// the permissions that matter for that channel.
#[derive(Debug)]
struct ChannelOverwrites {
    everyone: OverwritePair,
    roles: HashMap<Id<RoleMarker>, OverwritePair>,
    members: HashMap<Id<UserMarker>, OverwritePair>,
}

impl ChannelOverwrites {
    fn new(
        overwrites: &[PermissionOverwrite],
        everyone_role_id: Id<RoleMarker>,
        relevant: Permissions,
    ) -> Self {
        let mut this = Self {
            everyone: NO_OVERWRITE,
            roles: HashMap::new(),
            members: HashMap::new(),
        };
        for ow in overwrites {
            let pair = (ow.allow & relevant, ow.deny & relevant);
            match ow.kind {
                PermissionOverwriteType::Role if ow.id.cast() == everyone_role_id => {
                    this.everyone = pair;
                }
                PermissionOverwriteType::Role => {
                    this.roles.insert(ow.id.cast(), pair);
                }
                PermissionOverwriteType::Member => {
                    this.members.insert(ow.id.cast(), pair);
                }
                // Refused by `has_unknown_overwrite` before any of this is built.
                _ => {}
            }
        }
        this
    }

    fn role(&self, role_id: Id<RoleMarker>) -> OverwritePair {
        self.roles.get(&role_id).copied().unwrap_or(NO_OVERWRITE)
    }

    fn member(&self, user_id: Id<UserMarker>) -> OverwritePair {
        self.members.get(&user_id).copied().unwrap_or(NO_OVERWRITE)
    }
}

/// What a set of roles contributes to a member's permissions in the source and
/// target channels.
///
/// Discord merges a member's roles by OR-ing their base permissions, and their
/// role overwrites by OR-ing the allows and the denies separately. Every field
/// here is merged the same way, so the grant of any set of roles is the
/// [`Self::union`] of the grants of its roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RoleGrant {
    base: Permissions,
    source: OverwritePair,
    target: OverwritePair,
}

impl RoleGrant {
    fn union(self, other: Self) -> Self {
        let merge = |(a1, d1): OverwritePair, (a2, d2): OverwritePair| (a1 | a2, d1 | d2);
        Self {
            base: self.base | other.base,
            source: merge(self.source, other.source),
            target: merge(self.target, other.target),
        }
    }
}

/// Applies Discord's permission algorithm to one channel.
///
/// `ADMINISTRATOR` in the base grants everything. Otherwise the `@everyone`
/// overwrite, the merged role overwrites and the member's own overwrite are
/// applied in that order, each as deny-then-allow.
fn channel_permissions(
    base: Permissions,
    everyone: OverwritePair,
    roles: OverwritePair,
    member: OverwritePair,
) -> Permissions {
    if base.contains(Permissions::ADMINISTRATOR) {
        return Permissions::all();
    }
    [everyone, roles, member]
        .into_iter()
        .fold(base, |perms, (allow, deny)| (perms - deny) | allow)
}

/// Decides whether everyone who can read the source channel could also read the
/// target channel.
///
/// Members are split by whether either channel carries an overwrite for them
/// personally:
///
/// - Without one, a member's access depends only on which roles they hold.
///   [`Self::roles_preserve_visibility`] checks every combination of roles a
///   member could hold — not just each role on its own, since Discord merges a
///   member's role overwrites before applying them.
/// - With one, the member is looked up and judged on their actual roles by
///   [`Self::member_preserves_visibility`]. [`Self::members_to_verify`] lists
///   only those whose personal overwrite could make a difference.
#[derive(Debug)]
struct Visibility {
    source: ChannelOverwrites,
    target: ChannelOverwrites,
    /// What a member needs in the target, from [`read_target_requirement`].
    read_target: Permissions,
    everyone_role_id: Id<RoleMarker>,
    role_perms: RolePermissions,
    /// Every grant a member without personal overwrites can hold.
    reachable: HashSet<RoleGrant>,
}

impl Visibility {
    fn new(
        source_overwrites: &[PermissionOverwrite],
        target_overwrites: &[PermissionOverwrite],
        read_target: Permissions,
        role_perms: RolePermissions,
        everyone_role_id: Id<RoleMarker>,
    ) -> Self {
        let mut this = Self {
            source: ChannelOverwrites::new(source_overwrites, everyone_role_id, READ_SOURCE),
            target: ChannelOverwrites::new(target_overwrites, everyone_role_id, read_target),
            read_target,
            everyone_role_id,
            role_perms,
            reachable: HashSet::new(),
        };
        this.reachable = this.reachable_grants();
        this
    }

    /// The grant of `@everyone`, which every member holds. Its overwrite is
    /// applied separately, so it contributes base permissions only.
    fn everyone_grant(&self) -> RoleGrant {
        RoleGrant {
            base: self.base_of(self.everyone_role_id),
            source: NO_OVERWRITE,
            target: NO_OVERWRITE,
        }
    }

    fn role_grant(&self, role_id: Id<RoleMarker>) -> RoleGrant {
        RoleGrant {
            base: self.base_of(role_id),
            source: self.source.role(role_id),
            target: self.target.role(role_id),
        }
    }

    /// Masked to the base permissions that can change the outcome of
    /// [`channel_permissions`], so that equivalent roles collapse into one
    /// [`RoleGrant`] and [`Self::reachable_grants`] stays small.
    fn base_of(&self, role_id: Id<RoleMarker>) -> Permissions {
        self.role_perms
            .get(&role_id)
            .copied()
            .unwrap_or_else(Permissions::empty)
            & (Permissions::ADMINISTRATOR | READ_SOURCE | self.read_target)
    }

    /// Closes `@everyone`'s grant under union with every other role's grant.
    ///
    /// Why not enumerate role subsets: that is exponential in the number of
    /// roles, while the masked grants have only a few bits, so the closure stays
    /// small no matter how many roles the guild has.
    ///
    /// Roles named only by an overwrite are included too: a role created after
    /// the role cache was filled still restricts whoever holds it.
    fn reachable_grants(&self) -> HashSet<RoleGrant> {
        let roles: HashSet<RoleGrant> = self
            .role_perms
            .keys()
            .chain(self.source.roles.keys())
            .chain(self.target.roles.keys())
            .filter(|&&id| id != self.everyone_role_id)
            .map(|&id| self.role_grant(id))
            .collect();

        let start = self.everyone_grant();
        let mut reachable = HashSet::from([start]);
        let mut pending = vec![start];
        while let Some(grant) = pending.pop() {
            for &role in &roles {
                let next = grant.union(role);
                if reachable.insert(next) {
                    pending.push(next);
                }
            }
        }
        reachable
    }

    fn source_permissions(&self, grant: RoleGrant, member: OverwritePair) -> Permissions {
        channel_permissions(grant.base, self.source.everyone, grant.source, member)
    }

    fn target_permissions(&self, grant: RoleGrant, member: OverwritePair) -> Permissions {
        channel_permissions(grant.base, self.target.everyone, grant.target, member)
    }

    /// Whether a member with `grant` and the given personal overwrites can read
    /// the source but not the target.
    fn leaks(
        &self,
        grant: RoleGrant,
        source_member: OverwritePair,
        target_member: OverwritePair,
    ) -> bool {
        self.source_permissions(grant, source_member)
            .contains(READ_SOURCE)
            && !self
                .target_permissions(grant, target_member)
                .contains(self.read_target)
    }

    /// Whether no combination of roles lets a member read the source but not the
    /// target, for members without personal overwrites.
    fn roles_preserve_visibility(&self) -> bool {
        self.reachable
            .iter()
            .all(|&grant| !self.leaks(grant, NO_OVERWRITE, NO_OVERWRITE))
    }

    /// Members whose personal overwrites could let them read the source but not
    /// the target, sorted by id.
    ///
    /// A member is listed only when some role combination, together with their
    /// personal overwrites, leaks; for anyone else the outcome is the same
    /// whatever roles they actually hold, so looking them up is wasted.
    fn members_to_verify(&self) -> Vec<Id<UserMarker>> {
        let users: HashSet<Id<UserMarker>> = self
            .source
            .members
            .keys()
            .chain(self.target.members.keys())
            .copied()
            .collect();
        let mut users: Vec<_> = users
            .into_iter()
            .filter(|&user| {
                let (source_member, target_member) =
                    (self.source.member(user), self.target.member(user));
                self.reachable
                    .iter()
                    .any(|&grant| self.leaks(grant, source_member, target_member))
            })
            .collect();
        users.sort_unstable();
        users
    }

    /// Whether `user`, holding `roles`, cannot read the source without also being
    /// able to read the target.
    fn member_preserves_visibility(&self, user: Id<UserMarker>, roles: &[Id<RoleMarker>]) -> bool {
        let grant = roles
            .iter()
            .filter(|&&id| id != self.everyone_role_id)
            .fold(self.everyone_grant(), |grant, &id| {
                grant.union(self.role_grant(id))
            });
        !self.leaks(grant, self.source.member(user), self.target.member(user))
    }
}

/// Resolves the channel that carries the properties a thread inherits — its
/// permission overwrites and its NSFW flag.
///
/// Threads hold neither of their own, so for any thread the parent channel is
/// fetched and returned. Non-thread channels are returned unchanged. A thread
/// without a `parent_id` is treated as an error.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn permission_channel(
    channel: &Channel,
    guild_id: Id<GuildMarker>,
    ctx: &BotContext,
) -> Result<Channel, PreviewError> {
    if !is_thread(channel.kind) {
        return Ok(channel.clone());
    }

    let parent_id = channel.parent_id.ok_or(PreviewError::Permission)?;
    CacheArgs {
        guild_id,
        channel_id: parent_id,
    }
    .get(&ctx.http)
    .await
    .map_err(|_| PreviewError::Cache)
}

/// Most members [`check_visibility`] looks up for a single link.
///
/// Each lookup is an API request made on every expansion, since the result
/// cannot be cached (see [`member_roles`]). Past this many, the link is refused
/// rather than spending that much rate limit on it.
const MAX_MEMBER_LOOKUPS: usize = 10;

/// Fetches the roles of `user_id`, or `None` when they are not in the guild.
///
/// Why not cached: without the privileged `GUILD_MEMBERS` intent no event
/// reports a member's role changes, so a cached entry could never be
/// invalidated.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn member_roles(
    ctx: &BotContext,
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
) -> Result<Option<Vec<Id<RoleMarker>>>, PreviewError> {
    match ctx.http.guild_member(guild_id, user_id).await {
        Ok(response) => Ok(Some(response.model().await?.roles)),
        Err(e) if matches!(e.kind(), ErrorType::Response { status, .. } if status.get() == 404) => {
            Ok(None)
        }
        Err(e) => Err(e.into()),
    }
}

/// Validates that everyone who can read `source_channel` could also read the
/// linked message in `channel`.
///
/// The expanded content is posted as a single message that all members of
/// `source_channel` can read, so the linked channel must be at least as
/// readable as the source channel to avoid leaking restricted content.
/// "Readable" means `VIEW_CHANNEL` on the source, and on the target what
/// [`read_target_requirement`] asks for. Permissions are evaluated by
/// [`Visibility`] the way Discord does, so roles, role combinations, per-member
/// overwrites and `ADMINISTRATOR` need no special cases.
///
/// What remains is refused because it cannot be evaluated:
///
/// - Links into another guild, refused by [`Preview::get`] before this runs:
///   roles and overwrites are guild-local, and the bot may not be in that guild.
/// - Private threads (and DMs): their readers are the thread's members, and
///   listing them needs the `GUILD_MEMBERS` intent.
/// - Overwrites of a kind this library does not know.
/// - More than [`MAX_MEMBER_LOOKUPS`] members needing a lookup.
///
/// Threads have no overwrites of their own and are judged by their parent. For
/// a public thread that is exact; for a private thread used as the source it
/// over-approximates the readers, which can only cause a refusal.
/// `dest_perm` is `channel` resolved by [`permission_channel`].
#[cfg_attr(coverage_nightly, coverage(off))]
async fn check_visibility(
    channel: &Channel,
    dest_perm: &Channel,
    source_channel: &Channel,
    guild_id: Id<GuildMarker>,
    ctx: &BotContext,
) -> Result<(), PreviewError> {
    if matches!(
        channel.kind,
        ChannelType::PrivateThread | ChannelType::Private
    ) {
        tracing::debug!(kind = ?channel.kind, "rejected: private channel or thread");
        return Err(PreviewError::Permission);
    }

    let source_perm = permission_channel(source_channel, guild_id, ctx).await?;

    if has_unknown_overwrite(overwrites(dest_perm))
        || has_unknown_overwrite(overwrites(&source_perm))
    {
        tracing::debug!("rejected: overwrite of unknown kind");
        return Err(PreviewError::Permission);
    }

    let role_perms = crate::cache::role_permissions(&ctx.http, guild_id)
        .await
        .map_err(|_| PreviewError::Cache)?;
    let visibility = Visibility::new(
        overwrites(&source_perm),
        overwrites(dest_perm),
        read_target_requirement(channel.kind),
        role_perms,
        guild_id.cast(),
    );

    if !visibility.roles_preserve_visibility() {
        tracing::debug!("rejected: a role combination can read the source but not the target");
        return Err(PreviewError::Permission);
    }

    let users = visibility.members_to_verify();
    if users.len() > MAX_MEMBER_LOOKUPS {
        tracing::debug!(
            members = users.len(),
            "rejected: too many members with personal overwrites to verify"
        );
        return Err(PreviewError::Permission);
    }

    let roles = try_join_all(users.iter().map(|&user| member_roles(ctx, guild_id, user))).await?;
    for (&user, roles) in users.iter().zip(roles) {
        if let Some(roles) = roles
            && !visibility.member_preserves_visibility(user, &roles)
        {
            tracing::debug!(%user, "rejected: member can read the source but not the target");
            return Err(PreviewError::Permission);
        }
    }

    Ok(())
}

impl Preview {
    /// Retrieves a preview for the given message link.
    ///
    /// Every rule deciding whether a link may be expanded lives here. In order,
    /// the link must point into `guild_id` — the guild of `source_channel` —
    /// and the linked channel must not be NSFW, must not be a private thread or
    /// DM, and must be readable by everyone who can view `source_channel`. The expanded
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
        skip(args, ctx, guild_id, source_channel),
        fields(
            guild_id = %args.guild_id,
            channel_id = %args.channel_id,
            message_id = %args.message_id,
        )
    )]
    async fn get(
        args: &MessageLinkIDs,
        ctx: &BotContext,
        guild_id: Id<GuildMarker>,
        source_channel: &Channel,
    ) -> Result<Preview, PreviewError> {
        if is_cross_guild(args.guild_id, guild_id) {
            tracing::debug!(link_guild_id = %args.guild_id, "rejected: cross-guild link");
            return Err(PreviewError::CrossGuild);
        }

        let caches = CacheArgs {
            // Not `args.guild_id`: that is the value the URL claims. It equals
            // the source guild after the check above, so take the one Discord
            // reported for the request and keep guild scoping sourced from it.
            guild_id,
            channel_id: args.channel_id,
        };

        let channel = caches
            .get(&ctx.http)
            .await
            .map_err(|_| PreviewError::Cache)?;
        tracing::debug!(kind = ?channel.kind, nsfw = ?channel.nsfw, "resolved target channel");

        // Judged on the parent for threads, not on `channel.nsfw` directly:
        // Discord omits `nsfw` from thread objects because threads inherit it,
        // and twilight leaves the absent field `None`, so every thread under an
        // NSFW channel would otherwise slip past this gate.
        let age_gate = permission_channel(&channel, guild_id, ctx).await?;
        if age_gate.nsfw.unwrap_or(false) {
            tracing::debug!("rejected: target channel is NSFW");
            return Err(PreviewError::Nsfw);
        }

        // When the link points to the same channel the request came from, the
        // expansion is posted back into that very channel. Every member who can
        // read the reply can already read the original message, so there is
        // nothing to leak and the visibility checks can be skipped.
        if requires_visibility_check(args.channel_id, source_channel.id) {
            check_visibility(&channel, &age_gate, source_channel, guild_id, ctx).await?;
        }

        let started = std::time::Instant::now();
        let message = ctx
            .http
            .message(channel.id, args.message_id)
            .await?
            .model()
            .await?;
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
        assert_eq!(results[0].guild_id, Id::new(123456789));
        assert_eq!(results[0].channel_id, Id::new(987654321));
        assert_eq!(results[0].message_id, Id::new(111111111));
    }

    #[test]
    fn parse_ptb_link() {
        let text = "https://ptb.discord.com/channels/123/456/789";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].guild_id, Id::new(123));
    }

    #[test]
    fn parse_canary_link() {
        let text = "https://canary.discord.com/channels/123/456/789";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].guild_id, Id::new(123));
    }

    #[test]
    fn parse_multiple_links() {
        let text = "https://discord.com/channels/1/2/3 and https://discord.com/channels/4/5/6";
        let results = MessageLinkIDs::parse_all(text);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].guild_id, Id::new(1));
        assert_eq!(results[1].guild_id, Id::new(4));
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
    fn parse_ignores_zero_ids() {
        for text in [
            "https://discord.com/channels/0/1/2",
            "https://discord.com/channels/1/0/2",
            "https://discord.com/channels/1/2/0",
        ] {
            assert!(MessageLinkIDs::parse_all(text).is_empty(), "{text}");
        }
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
        assert_eq!(results[0].message_id, Id::new(3));
    }

    // --- Privacy / permission resolution ---

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
            id: Id::new(id),
            kind: PermissionOverwriteType::Role,
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
            id: Id::new(id),
            kind: PermissionOverwriteType::Member,
        }
    }

    const EVERYONE: Id<RoleMarker> = Id::new(1);
    const MEMBER: Id<RoleMarker> = Id::new(100);
    const SPECIAL: Id<RoleMarker> = Id::new(200);
    const ADMIN: Id<RoleMarker> = Id::new(300);

    #[test]
    fn thread_kinds_detected() {
        assert!(is_thread(ChannelType::PublicThread));
        assert!(is_thread(ChannelType::AnnouncementThread));
        assert!(is_thread(ChannelType::PrivateThread));
        assert!(!is_thread(ChannelType::GuildText));
        assert!(!is_thread(ChannelType::GuildVoice));
    }

    #[test]
    fn unknown_overwrite_kind_is_detected() {
        let unknown = PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::empty(),
            id: Id::new(5),
            kind: PermissionOverwriteType::Unknown(9),
        };
        assert!(has_unknown_overwrite(&[role_ow(1, true, false), unknown]));
        assert!(!has_unknown_overwrite(&[
            role_ow(1, true, false),
            member_ow(5, false, true)
        ]));
        assert!(!has_unknown_overwrite(&[]));
    }

    #[test]
    fn same_channel_skips_visibility_check() {
        let chan = Id::new(42);
        // Quoting within the same channel needs no visibility check.
        assert!(!requires_visibility_check(chan, chan));
        // A link to a different channel still requires validation.
        assert!(requires_visibility_check(chan, Id::new(99)));
    }

    #[test]
    fn cross_guild_links_are_rejected() {
        let guild = Id::new(7);
        // A link into the guild it was posted in may be judged further.
        assert!(!is_cross_guild(guild, guild));
        // A link from any other guild is refused outright.
        assert!(is_cross_guild(Id::new(8), guild));
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
            !PreviewError::Discord(Box::new(std::io::Error::other("boom"))).is_policy_rejection()
        );
    }

    // --- Visibility ---

    const ROLE_A: Id<RoleMarker> = Id::new(400);
    const ROLE_B: Id<RoleMarker> = Id::new(500);
    const USER: Id<UserMarker> = Id::new(5);

    /// Builds an overwrite with arbitrary permissions.
    fn overwrite(
        kind: PermissionOverwriteType,
        id: u64,
        allow: Permissions,
        deny: Permissions,
    ) -> PermissionOverwrite {
        PermissionOverwrite {
            allow,
            deny,
            id: Id::new(id),
            kind,
        }
    }

    /// Role permissions where `@everyone` can view channels and read their history
    /// by default, as in a freshly created guild, and every other role adds nothing.
    fn default_roles(others: &[Id<RoleMarker>]) -> HashMap<Id<RoleMarker>, Permissions> {
        let mut roles = HashMap::from([(EVERYONE, READ_TARGET)]);
        roles.extend(others.iter().map(|&id| (id, Permissions::empty())));
        roles
    }

    fn visibility(
        source: &[PermissionOverwrite],
        target: &[PermissionOverwrite],
        roles: &HashMap<Id<RoleMarker>, Permissions>,
    ) -> Visibility {
        Visibility::new(source, target, READ_TARGET, roles.clone(), EVERYONE)
    }

    /// `@everyone` denied, `role` allowed: a channel only `role` can view.
    fn gated_to(role: Id<RoleMarker>) -> [PermissionOverwrite; 2] {
        [
            role_ow(EVERYONE.get(), false, true),
            role_ow(role.get(), true, false),
        ]
    }

    #[test]
    fn public_channels_preserve_visibility() {
        let v = visibility(&[], &[], &default_roles(&[MEMBER]));
        assert!(v.roles_preserve_visibility());
        assert!(v.members_to_verify().is_empty());
    }

    #[test]
    fn equally_gated_channels_preserve_visibility() {
        let gate = gated_to(MEMBER);
        let v = visibility(&gate, &gate, &default_roles(&[MEMBER]));
        assert!(v.roles_preserve_visibility());
    }

    #[test]
    fn target_gated_to_another_role_is_rejected() {
        let v = visibility(
            &gated_to(MEMBER),
            &gated_to(SPECIAL),
            &default_roles(&[MEMBER, SPECIAL]),
        );
        assert!(!v.roles_preserve_visibility());
    }

    #[test]
    fn narrower_source_may_quote_wider_target() {
        let v = visibility(&gated_to(SPECIAL), &[], &default_roles(&[SPECIAL]));
        assert!(v.roles_preserve_visibility());
    }

    #[test]
    fn member_granted_source_by_one_role_and_denied_target_by_another_is_rejected() {
        // GHSA-9857-rwch-jhw6: each role alone views both channels or neither, but a
        // member holding A and B views the source (B's allow follows A's deny) and
        // not the target (A's deny is its only role overwrite).
        let source = [
            role_ow(ROLE_A.get(), false, true),
            role_ow(ROLE_B.get(), true, false),
        ];
        let target = [role_ow(ROLE_A.get(), false, true)];
        let v = visibility(&source, &target, &default_roles(&[ROLE_A, ROLE_B]));
        assert!(!v.roles_preserve_visibility());
    }

    #[test]
    fn role_denied_on_target_held_alongside_role_gating_source_is_rejected() {
        // A member holding A views the gated source; adding B, denied on the
        // otherwise public target, takes the target away.
        let target = [role_ow(ROLE_B.get(), false, true)];
        let v = visibility(
            &gated_to(ROLE_A),
            &target,
            &default_roles(&[ROLE_A, ROLE_B]),
        );
        assert!(!v.roles_preserve_visibility());
    }

    #[test]
    fn role_allow_on_target_outweighs_another_roles_deny() {
        // Role allows are applied after role denies, so holding B as well cannot
        // take the target away from a member who holds A.
        let target = [
            role_ow(ROLE_B.get(), false, true),
            role_ow(ROLE_A.get(), true, false),
        ];
        let v = visibility(
            &gated_to(ROLE_A),
            &target,
            &default_roles(&[ROLE_A, ROLE_B]),
        );
        assert!(v.roles_preserve_visibility());
    }

    #[test]
    fn target_without_read_message_history_is_rejected() {
        let target = [overwrite(
            PermissionOverwriteType::Role,
            EVERYONE.get(),
            Permissions::empty(),
            Permissions::READ_MESSAGE_HISTORY,
        )];
        let v = visibility(&[], &target, &default_roles(&[]));
        assert!(!v.roles_preserve_visibility());
    }

    #[test]
    fn source_without_read_message_history_still_counts_its_viewers() {
        // The reply is a new message, so viewing the source is enough to read it.
        let source = [overwrite(
            PermissionOverwriteType::Role,
            EVERYONE.get(),
            Permissions::empty(),
            Permissions::READ_MESSAGE_HISTORY,
        )];
        let restricted_target = gated_to(MEMBER);
        let v = visibility(&source, &restricted_target, &default_roles(&[MEMBER]));
        assert!(!v.roles_preserve_visibility());
    }

    #[test]
    fn voice_and_stage_targets_also_require_connect() {
        for kind in [ChannelType::GuildVoice, ChannelType::GuildStageVoice] {
            assert_eq!(
                read_target_requirement(kind),
                READ_TARGET | Permissions::CONNECT
            );
        }
        assert_eq!(read_target_requirement(ChannelType::GuildText), READ_TARGET);
        assert_eq!(
            read_target_requirement(ChannelType::PublicThread),
            READ_TARGET
        );
    }

    #[test]
    fn voice_target_viewable_but_not_connectable_is_rejected() {
        // A locked voice channel: everyone sees it, only ROLE_A may connect and
        // so read its text chat.
        let mut roles = default_roles(&[ROLE_A]);
        roles.insert(EVERYONE, READ_TARGET | Permissions::CONNECT);
        let target = [
            overwrite(
                PermissionOverwriteType::Role,
                EVERYONE.get(),
                Permissions::empty(),
                Permissions::CONNECT,
            ),
            overwrite(
                PermissionOverwriteType::Role,
                ROLE_A.get(),
                Permissions::CONNECT,
                Permissions::empty(),
            ),
        ];
        let read_voice = read_target_requirement(ChannelType::GuildVoice);
        let v = Visibility::new(&[], &target, read_voice, roles.clone(), EVERYONE);
        assert!(!v.roles_preserve_visibility());

        let v = Visibility::new(&gated_to(ROLE_A), &target, read_voice, roles, EVERYONE);
        assert!(v.roles_preserve_visibility());
    }

    #[test]
    fn role_missing_from_role_permissions_still_restricts_its_holders() {
        // ROLE_A is newer than the cached role permissions but already denied on
        // the target.
        let target = [role_ow(ROLE_A.get(), false, true)];
        let v = visibility(&[], &target, &default_roles(&[]));
        assert!(!v.roles_preserve_visibility());
    }

    #[test]
    fn administrator_reads_any_target() {
        let mut roles = default_roles(&[]);
        roles.insert(ADMIN, Permissions::ADMINISTRATOR);
        let nobody = [role_ow(EVERYONE.get(), false, true)];
        // Only administrators view the source, and they read everything.
        let v = visibility(&nobody, &[role_ow(ADMIN.get(), false, true)], &roles);
        assert!(v.roles_preserve_visibility());
    }

    #[test]
    fn missing_everyone_role_grants_no_base_permissions() {
        // Nobody can view the source without a base grant, so nothing can leak.
        let roles = HashMap::from([(MEMBER, Permissions::empty())]);
        let v = visibility(&[], &gated_to(SPECIAL), &roles);
        assert!(v.roles_preserve_visibility());
    }

    #[test]
    fn reachable_grants_stay_bounded_with_many_roles() {
        let ids: Vec<Id<RoleMarker>> = (1000..1200).map(Id::new).collect();
        let source: Vec<_> = ids
            .iter()
            .step_by(2)
            .map(|id| role_ow(id.get(), true, false))
            .collect();
        let target: Vec<_> = ids
            .iter()
            .step_by(3)
            .map(|id| role_ow(id.get(), false, true))
            .collect();
        let v = visibility(&source, &target, &default_roles(&ids));
        // Base, source and target bits: 3 + 2 + 4.
        assert!(v.reachable.len() <= 1 << 9);
    }

    #[test]
    fn member_granted_source_quoting_public_target_needs_no_lookup() {
        let source = [
            role_ow(EVERYONE.get(), false, true),
            member_ow(USER.get(), true, false),
        ];
        let v = visibility(&source, &[], &default_roles(&[MEMBER]));
        assert!(v.roles_preserve_visibility());
        assert!(v.members_to_verify().is_empty());
    }

    #[test]
    fn member_granted_source_quoting_restricted_target_looks_the_member_up() {
        let source = [
            role_ow(EVERYONE.get(), false, true),
            member_ow(USER.get(), true, false),
        ];
        let v = visibility(&source, &gated_to(MEMBER), &default_roles(&[MEMBER]));
        assert!(v.roles_preserve_visibility());
        assert_eq!(v.members_to_verify(), [USER]);

        // Judged on the roles they actually hold.
        assert!(v.member_preserves_visibility(USER, &[MEMBER]));
        assert!(!v.member_preserves_visibility(USER, &[]));
    }

    #[test]
    fn member_denied_on_target_is_looked_up_and_rejected_if_they_view_the_source() {
        let target = [member_ow(USER.get(), false, true)];
        let v = visibility(&[], &target, &default_roles(&[]));
        assert_eq!(v.members_to_verify(), [USER]);
        assert!(!v.member_preserves_visibility(USER, &[]));
    }

    #[test]
    fn member_denied_on_target_who_cannot_view_the_source_is_fine() {
        let v = visibility(
            &gated_to(MEMBER),
            &[member_ow(USER.get(), false, true)],
            &default_roles(&[MEMBER]),
        );
        assert_eq!(v.members_to_verify(), [USER]);
        assert!(v.member_preserves_visibility(USER, &[]));
        assert!(!v.member_preserves_visibility(USER, &[MEMBER]));
    }

    #[test]
    fn member_shut_out_of_source_needs_no_lookup() {
        let source = [member_ow(USER.get(), false, true)];
        let target = [member_ow(USER.get(), false, true)];
        let v = visibility(&source, &target, &default_roles(&[]));
        assert!(v.members_to_verify().is_empty());
    }

    #[test]
    fn member_allowed_only_on_target_needs_no_lookup() {
        let v = visibility(
            &gated_to(MEMBER),
            &[
                role_ow(EVERYONE.get(), false, true),
                member_ow(USER.get(), true, false),
            ],
            &default_roles(&[MEMBER]),
        );
        // MEMBER views the source but not the target, whatever USER can do.
        assert!(!v.roles_preserve_visibility());
        assert!(v.members_to_verify().is_empty());
    }

    #[test]
    fn member_granted_both_channels_needs_no_lookup() {
        // A restricted channel quoted from one made private by adding the same
        // members individually: their personal grant already covers the target.
        let source = [
            role_ow(EVERYONE.get(), false, true),
            member_ow(USER.get(), true, false),
        ];
        let target = [
            role_ow(EVERYONE.get(), false, true),
            member_ow(USER.get(), true, false),
        ];
        let v = visibility(&source, &target, &default_roles(&[MEMBER]));
        assert!(v.roles_preserve_visibility());
        assert!(v.members_to_verify().is_empty());
    }

    #[test]
    fn member_overwrite_outweighs_role_overwrites() {
        // A personal allow is applied last, so it restores what a role denied.
        let target = [
            role_ow(MEMBER.get(), false, true),
            overwrite(
                PermissionOverwriteType::Member,
                USER.get(),
                READ_TARGET,
                Permissions::empty(),
            ),
        ];
        let v = visibility(
            &[member_ow(USER.get(), true, false)],
            &target,
            &default_roles(&[MEMBER]),
        );
        assert!(v.member_preserves_visibility(USER, &[MEMBER]));
    }

    // --- Preview embed rendering ---

    // `Message` and `Channel` have no `Default` and dozens of fields, so build
    // them from JSON.
    fn message_with(avatar: Option<&str>) -> Message {
        serde_json::from_value(serde_json::json!({
            "id": "1",
            "channel_id": "2",
            "type": 0,
            "author": {"id": "3", "username": "author", "discriminator": "0", "avatar": avatar},
            "content": "quoted content",
            "timestamp": "2024-01-01T00:00:00+00:00",
            "edited_timestamp": null,
            "tts": false,
            "mention_everyone": false,
            "mentions": [],
            "mention_roles": [],
            "attachments": [],
            "embeds": [],
            "pinned": false,
        }))
        .unwrap()
    }

    fn channel_named(name: Option<&str>) -> Channel {
        serde_json::from_value(serde_json::json!({"id": "2", "type": 0, "name": name})).unwrap()
    }

    #[test]
    fn embed_carries_the_quoted_message_and_its_origin() {
        let message = message_with(None);

        let embed = preview_embed(&message, &channel_named(Some("general")));

        assert_eq!(
            embed,
            EmbedBuilder::new()
                .description("quoted content")
                .author(EmbedAuthorBuilder::new("author"))
                .timestamp(message.timestamp)
                .color(PREVIEW_EMBED_COLOUR)
                .footer(EmbedFooterBuilder::new("general"))
                .build()
        );
    }

    #[test]
    fn a_channel_without_a_name_gets_no_footer() {
        let embed = preview_embed(&message_with(None), &channel_named(None));

        assert!(embed.footer.is_none());
    }

    #[test]
    fn an_author_without_an_avatar_gets_no_icon_url() {
        let channel = channel_named(Some("general"));
        let without_avatar = preview_embed(&message_with(None), &channel);

        let with_avatar = preview_embed(
            &message_with(Some("a_00000000000000000000000000000000")),
            &channel,
        );

        assert_ne!(without_avatar, with_avatar);
        assert!(format!("{with_avatar:?}").contains("a_00000000000000000000000000000000"));
    }

    fn attachment(url: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "4",
            "filename": "image.png",
            "proxy_url": url,
            "size": 1,
            "url": url,
        })
    }

    #[test]
    fn the_first_attachment_becomes_the_embed_image() {
        let mut message = message_with(None);
        message.attachments = serde_json::from_value(serde_json::json!([
            attachment("https://cdn.discordapp.com/attachments/1/2/first.png"),
            attachment("https://cdn.discordapp.com/attachments/1/2/second.png"),
        ]))
        .unwrap();

        let embed = preview_embed(&message, &channel_named(Some("general")));

        assert_eq!(
            embed.image.map(|image| image.url),
            Some("https://cdn.discordapp.com/attachments/1/2/first.png".to_string())
        );
    }

    #[test]
    fn a_message_without_attachments_gets_no_embed_image() {
        let embed = preview_embed(&message_with(None), &channel_named(Some("general")));

        assert!(embed.image.is_none());
    }

    #[test]
    fn a_static_avatar_is_served_as_webp() {
        let author = message_with(Some("00000000000000000000000000000000")).author;

        assert_eq!(
            avatar_url(&author).as_deref(),
            Some(
                "https://cdn.discordapp.com/avatars/3/00000000000000000000000000000000.webp?size=1024"
            )
        );
    }

    #[test]
    fn an_animated_avatar_is_served_as_gif() {
        let author = message_with(Some("a_00000000000000000000000000000000")).author;

        assert_eq!(
            avatar_url(&author).as_deref(),
            Some(
                "https://cdn.discordapp.com/avatars/3/a_00000000000000000000000000000000.gif?size=1024"
            )
        );
    }

    #[test]
    fn an_author_without_an_avatar_has_no_avatar_url() {
        assert_eq!(avatar_url(&message_with(None).author), None);
    }

    #[test]
    fn a_channel_without_overwrites_has_none() {
        assert!(overwrites(&channel_named(Some("general"))).is_empty());
    }

    #[test]
    fn a_channel_exposes_its_overwrites() {
        let channel: Channel = serde_json::from_value(serde_json::json!({
            "id": "2",
            "type": 0,
            "permission_overwrites": [
                {"id": "1", "type": 0, "allow": "0", "deny": "1024"},
            ],
        }))
        .unwrap();

        assert_eq!(overwrites(&channel), [role_ow(1, false, true)]);
    }
}
