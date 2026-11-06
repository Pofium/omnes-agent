//! Channel configuration: the `[channels]` section - the top-level
//! `ChannelsConfig` instance tables plus the per-channel typed configs
//! (Telegram, Discord, Slack, Matrix, Lark, LINE, ...).
//!
//! Extracted from the `schema` module; re-exported with `pub use` so every
//! `crate::schema::<ChannelsType>` path is unchanged.

use super::*;

// Private helper used only by the channel configs below; kept in this module
// rather than `schema` since every `impl_reply_pacing!` invocation lives here.
macro_rules! impl_reply_pacing {
    ($($ty:ty),+ $(,)?) => {
        $(impl HasReplyPacing for $ty {
            fn reply_min_interval_secs(&self) -> u64 { self.reply_min_interval_secs }
            fn reply_queue_depth_max(&self) -> u16 { self.reply_queue_depth_max }
        })+
    };
}

// ── Channels ─────────────────────────────────────────────────────

/// Top-level channel configurations (`[channels]` section).
///
/// each channel type is a keyed table of named instances (aliases).
/// `[channels.telegram.default]` is the conventional single-instance key.
/// Access via `config.channels.telegram.get("default")`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels"]
pub struct ChannelsConfig {
    /// Enable the CLI interactive channel. Default: `true`.
    #[serde(default = "default_true")]
    pub cli: bool,
    /// Telegram bot channel instances (`[channels.telegram.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub telegram: HashMap<String, TelegramConfig>,
    /// Discord bot channel instances (`[channels.discord.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub discord: HashMap<String, DiscordConfig>,
    /// Slack bot channel instances (`[channels.slack.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub slack: HashMap<String, SlackConfig>,
    /// Mattermost bot channel instances (`[channels.mattermost.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub mattermost: HashMap<String, MattermostConfig>,
    /// Webhook channel instances (`[channels.webhook.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub webhook: HashMap<String, WebhookConfig>,
    /// iMessage channel instances (`[channels.imessage.<alias>]`, macOS only).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub imessage: HashMap<String, IMessageConfig>,
    /// Matrix channel instances (`[channels.matrix.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub matrix: HashMap<String, MatrixConfig>,
    /// Signal channel instances (`[channels.signal.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub signal: HashMap<String, SignalConfig>,
    /// WhatsApp channel instances (`[channels.whatsapp.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub whatsapp: HashMap<String, WhatsAppConfig>,
    /// Linq Partner API channel instances (`[channels.linq.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub linq: HashMap<String, LinqConfig>,
    /// Nextcloud Talk bot channel instances (`[channels.nextcloud_talk.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub nextcloud_talk: HashMap<String, NextcloudTalkConfig>,
    /// Email channel instances (`[channels.email.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub email: HashMap<String, crate::scattered_types::EmailConfig>,
    /// Gmail Pub/Sub push notification channel instances (`[channels.gmail_push.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub gmail_push: HashMap<String, crate::scattered_types::GmailPushConfig>,
    /// IRC channel instances (`[channels.irc.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub irc: HashMap<String, IrcConfig>,
    /// Twitch chat channel instances (`[channels.twitch.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub twitch: HashMap<String, TwitchConfig>,
    /// Lark channel instances (`[channels.lark.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub lark: HashMap<String, LarkConfig>,
    /// LINE Messaging API channel instances (`[channels.line.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub line: HashMap<String, LineConfig>,
    /// DingTalk channel instances (`[channels.dingtalk.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub dingtalk: HashMap<String, DingTalkConfig>,
    /// WeCom (WeChat Enterprise) Bot Webhook channel instances (`[channels.wecom.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub wecom: HashMap<String, WeComConfig>,
    /// WeCom AI Bot WebSocket channel instances (`[channels.wecom_ws.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub wecom_ws: HashMap<String, WeComWsConfig>,
    /// WeChat personal iLink Bot channel instances (`[channels.wechat.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub wechat: HashMap<String, WeChatConfig>,
    /// QQ Official Bot channel instances (`[channels.qq.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub qq: HashMap<String, QQConfig>,
    /// X/Twitter channel instances (`[channels.twitter.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub twitter: HashMap<String, TwitterConfig>,
    /// Mochat customer service channel instances (`[channels.mochat.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub mochat: HashMap<String, MochatConfig>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub nostr: HashMap<String, NostrConfig>,
    /// ClawdTalk voice channel instances (`[channels.clawdtalk.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub clawdtalk: HashMap<String, crate::scattered_types::ClawdTalkConfig>,
    /// Reddit channel instances (`[channels.reddit.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub reddit: HashMap<String, RedditConfig>,
    /// Bluesky channel instances (`[channels.bluesky.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub bluesky: HashMap<String, BlueskyConfig>,
    /// Git-forge channel instances (`[channels.git.<alias>]`). GitHub is
    /// the first provider; the `provider` field selects the forge.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub git: HashMap<String, GitConfig>,
    /// Voice call channel instances (`[channels.voice_call.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub voice_call: HashMap<String, crate::scattered_types::VoiceCallConfig>,
    /// Voice wake word detection channel instances (`[channels.voice_wake.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub voice_wake: HashMap<String, VoiceWakeConfig>,
    /// Voice duplex instances (`[channels.voice_duplex.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub voice_duplex: HashMap<String, VoiceDuplexConfig>,
    /// MQTT channel instances (`[channels.mqtt.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub mqtt: HashMap<String, MqttConfig>,
    /// AMQP channel instances (`[channels.amqp.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub amqp: HashMap<String, AmqpConfig>,
    /// Filesystem SOP listener instances (`[channels.filesystem.<alias>]`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub filesystem: HashMap<String, FilesystemConfig>,
    /// WASM channel plugin instances (`[channels.plugin.<alias>]`).
    /// The declaration selects a package and logical binding only; operator
    /// values remain in the instance-keyed `plugins.entries` store.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[nested]
    pub plugin: HashMap<String, PluginChannelConfig>,
    /// Base timeout in seconds for processing a single channel message (LLM + tools).
    /// Runtime uses this as a per-turn budget that scales with tool-loop depth
    /// (up to 4x, capped) so one slow/retried model call does not consume the
    /// entire conversation budget.
    /// Default: 300s for on-device LLMs (Ollama) which are slower than cloud APIs.
    #[serde(default = "default_channel_message_timeout_secs")]
    pub message_timeout_secs: u64,
    /// Per-channel multiplier for the global channel message in-flight budget.
    /// Runtime multiplies this value by the configured channel count, then
    /// applies its global minimum and maximum bounds to one shared dispatcher
    /// semaphore. Default: `4`.
    #[serde(default = "default_channel_max_concurrent_per_channel")]
    pub max_concurrent_per_channel: usize,
    /// Whether to add acknowledgement reactions (👀 on receipt, ✅/⚠️ on
    /// completion) to incoming channel messages. Default: `true`.
    #[serde(default = "default_true")]
    pub ack_reactions: bool,
    /// Whether to send tool-call notification messages (e.g. `🔧 web_search_tool: …`)
    /// to channel users. When `false`, tool calls are still logged server-side but
    /// not forwarded as individual channel messages. Default: `false`.
    #[serde(default = "default_false")]
    pub show_tool_calls: bool,
    /// Persist channel conversation history to JSONL files so sessions survive
    /// daemon restarts. Files are stored in `{workspace}/sessions/`. Default: `true`.
    #[serde(default = "default_true")]
    pub session_persistence: bool,
    /// Session persistence backend: `"jsonl"` (legacy) or `"sqlite"` (new default).
    /// SQLite provides FTS5 search, metadata tracking, and TTL cleanup.
    #[serde(default = "default_session_backend")]
    pub session_backend: String,
    /// Auto-archive stale sessions older than this many hours. `0` disables. Default: `0`.
    #[serde(default)]
    pub session_ttl_hours: u32,
    /// Inbound message debounce window in milliseconds. When a sender fires
    /// multiple messages within this window, they are accumulated and dispatched
    /// as a single concatenated message. `0` disables debouncing. Default: `0`.
    #[serde(default)]
    pub debounce_ms: u64,
}

impl ChannelsConfig {
    /// Returns metadata and configuration status for every known channel type.
    ///
    /// Always returns the full set of channel types regardless of compile-time
    /// feature flags — the `configured` flag reflects whether the operator has
    /// populated that channel's config section. For a list restricted to only
    /// the channels compiled into this binary use
    /// `omnesagent_channels::listing::compiled_channels` instead.
    pub fn channels(&self) -> Vec<crate::traits::ChannelInfo> {
        use crate::traits::ChannelInfo;
        vec![
            ChannelInfo {
                kind: "telegram",
                name: "Telegram",
                desc: "connect your bot",
                configured: !self.telegram.is_empty(),
            },
            ChannelInfo {
                kind: "discord",
                name: "Discord",
                desc: "connect your bot",
                configured: !self.discord.is_empty(),
            },
            ChannelInfo {
                kind: "slack",
                name: "Slack",
                desc: "connect your bot",
                configured: !self.slack.is_empty(),
            },
            ChannelInfo {
                kind: "mattermost",
                name: "Mattermost",
                desc: "connect to your bot",
                configured: !self.mattermost.is_empty(),
            },
            ChannelInfo {
                kind: "imessage",
                name: "iMessage",
                desc: "macOS only",
                configured: !self.imessage.is_empty(),
            },
            ChannelInfo {
                kind: "matrix",
                name: "Matrix",
                desc: "self-hosted chat",
                configured: !self.matrix.is_empty(),
            },
            ChannelInfo {
                kind: "signal",
                name: "Signal",
                desc: "An open-source, encrypted messaging service",
                configured: !self.signal.is_empty(),
            },
            ChannelInfo {
                kind: "whatsapp",
                name: "WhatsApp",
                desc: "Business Cloud API",
                configured: !self.whatsapp.is_empty(),
            },
            ChannelInfo {
                kind: "whatsapp-web",
                name: "WhatsApp Web",
                desc: "native WhatsApp Web (wa-rs)",
                configured: self.whatsapp.values().any(|c| c.is_web_config()),
            },
            ChannelInfo {
                kind: "linq",
                name: "Linq",
                desc: "iMessage/RCS/SMS via Linq API",
                configured: !self.linq.is_empty(),
            },
            ChannelInfo {
                kind: "nextcloud",
                name: "NextCloud Talk",
                desc: "NextCloud Talk platform",
                configured: !self.nextcloud_talk.is_empty(),
            },
            ChannelInfo {
                kind: "email",
                name: "Email",
                desc: "Email over IMAP/SMTP",
                configured: !self.email.is_empty(),
            },
            ChannelInfo {
                kind: "gmail-push",
                name: "Gmail Push",
                desc: "Gmail Pub/Sub push notifications",
                configured: !self.gmail_push.is_empty(),
            },
            ChannelInfo {
                kind: "twitch",
                name: "Twitch",
                desc: "Twitch chat (IRC)",
                configured: !self.twitch.is_empty(),
            },
            ChannelInfo {
                kind: "irc",
                name: "IRC",
                desc: "IRC over TLS",
                configured: !self.irc.is_empty(),
            },
            ChannelInfo {
                kind: "lark",
                name: "Lark",
                desc: "Lark Bot",
                configured: !self.lark.is_empty(),
            },
            ChannelInfo {
                kind: "dingtalk",
                name: "DingTalk",
                desc: "DingTalk Stream Mode",
                configured: !self.dingtalk.is_empty(),
            },
            ChannelInfo {
                kind: "wecom",
                name: "WeCom",
                desc: "WeCom Bot Webhook",
                configured: !self.wecom.is_empty(),
            },
            ChannelInfo {
                kind: "wecom-ws",
                name: "WeCom WebSocket",
                desc: "WeCom AI Bot long connection",
                configured: !self.wecom_ws.is_empty(),
            },
            ChannelInfo {
                kind: "wechat",
                name: "WeChat",
                desc: "WeChat iLink Bot",
                configured: !self.wechat.is_empty(),
            },
            ChannelInfo {
                kind: "qq",
                name: "QQ Official",
                desc: "Tencent QQ Bot",
                configured: !self.qq.is_empty(),
            },
            ChannelInfo {
                kind: "nostr",
                name: "Nostr",
                desc: "Nostr DMs",
                configured: !self.nostr.is_empty(),
            },
            ChannelInfo {
                kind: "clawdtalk",
                name: "ClawdTalk",
                desc: "ClawdTalk Channel",
                configured: !self.clawdtalk.is_empty(),
            },
            ChannelInfo {
                kind: "reddit",
                name: "Reddit",
                desc: "Reddit bot (OAuth2)",
                configured: !self.reddit.is_empty(),
            },
            ChannelInfo {
                kind: "bluesky",
                name: "Bluesky",
                desc: "AT Protocol",
                configured: !self.bluesky.is_empty(),
            },
            ChannelInfo {
                kind: "git",
                name: "Git",
                desc: "Git forge (GitHub, Gitea, Forgejo): issues, PRs & events",
                configured: !self.git.is_empty(),
            },
            ChannelInfo {
                kind: "twitter",
                name: "X/Twitter",
                desc: "X/Twitter Bot via API v2",
                configured: !self.twitter.is_empty(),
            },
            ChannelInfo {
                kind: "mochat",
                name: "Mochat",
                desc: "Mochat Customer Service",
                configured: !self.mochat.is_empty(),
            },
            ChannelInfo {
                kind: "line",
                name: "LINE",
                desc: "connect your LINE bot",
                configured: !self.line.is_empty(),
            },
            ChannelInfo {
                kind: "voice-call",
                name: "Voice Call",
                desc: "outbound voice call channel",
                configured: !self.voice_call.is_empty(),
            },
            ChannelInfo {
                kind: "voice-wake",
                name: "VoiceWake",
                desc: "voice wake word detection",
                configured: !self.voice_wake.is_empty(),
            },
            ChannelInfo {
                kind: "mqtt",
                name: "MQTT",
                desc: "MQTT SOP Listener",
                configured: !self.mqtt.is_empty(),
            },
            ChannelInfo {
                kind: "amqp",
                name: "AMQP",
                desc: "AMQP topic consumer",
                configured: !self.amqp.is_empty(),
            },
            ChannelInfo {
                kind: "filesystem",
                name: "Filesystem",
                desc: "filesystem change SOP listener",
                configured: !self.filesystem.is_empty(),
            },
            ChannelInfo {
                kind: "webhook",
                name: "Webhook",
                desc: "HTTP endpoint",
                configured: !self.webhook.is_empty(),
            },
            ChannelInfo {
                kind: "plugin",
                name: "Plugin",
                desc: "installed WASM channel plugin",
                configured: !self.plugin.is_empty(),
            },
        ]
    }

    /// Returns `true` when at least one channel entry across all channel types
    /// has `enabled = true`. Used by the daemon to decide whether the channels
    /// supervisor should be started — a config with only `enabled = false`
    /// entries (e.g. partially-configured or disabled bots) must not start the
    /// supervisor, otherwise it exits immediately and restarts in a tight loop.
    pub fn has_any_enabled(&self) -> bool {
        self.telegram.values().any(|c| c.enabled)
            || self.discord.values().any(|c| c.enabled)
            || self.slack.values().any(|c| c.enabled)
            || self.mattermost.values().any(|c| c.enabled)
            || self.webhook.values().any(|c| c.enabled)
            || self.imessage.values().any(|c| c.enabled)
            || self.matrix.values().any(|c| c.enabled)
            || self.signal.values().any(|c| c.enabled)
            || self.whatsapp.values().any(|c| c.enabled)
            || self.linq.values().any(|c| c.enabled)
            || self.nextcloud_talk.values().any(|c| c.enabled)
            || self.email.values().any(|c| c.enabled)
            || self.gmail_push.values().any(|c| c.enabled)
            || self.irc.values().any(|c| c.enabled)
            || self.twitch.values().any(|c| c.enabled)
            || self.lark.values().any(|c| c.enabled)
            || self.line.values().any(|c| c.enabled)
            || self.dingtalk.values().any(|c| c.enabled)
            || self.wecom.values().any(|c| c.enabled)
            || self.wecom_ws.values().any(|c| c.enabled)
            || self.wechat.values().any(|c| c.enabled)
            || self.qq.values().any(|c| c.enabled)
            || self.twitter.values().any(|c| c.enabled)
            || self.mochat.values().any(|c| c.enabled)
            || self.nostr.values().any(|c| c.enabled)
            || self.clawdtalk.values().any(|c| c.enabled)
            || self.reddit.values().any(|c| c.enabled)
            || self.bluesky.values().any(|c| c.enabled)
            || self.voice_call.values().any(|c| c.enabled)
            || self.voice_wake.values().any(|c| c.enabled)
            || self.voice_duplex.values().any(|c| c.enabled)
            || self.mqtt.values().any(|c| c.enabled)
            || self.amqp.values().any(|c| c.enabled)
            || self.filesystem.values().any(|c| c.enabled)
            || self.git.values().any(|c| c.enabled)
            || self.plugin.values().any(|c| c.enabled)
    }

    /// One `(canonical_name, configured, deliverable)` row per channel in the
    /// registry. Single source for name-addressed channel lookups so no surface
    /// has to hardcode a subset of the channel list. `deliverable` is `false`
    /// for input-only transports whose `Channel::send` is a no-op (mqtt and
    /// amqp are fan-in listeners; voice_wake is input-only), so a name-addressed
    /// outbound surface such as `heartbeat.target` can refuse them at validation
    /// instead of accepting a target the delivery layer silently drops.
    pub fn channel_presence(&self) -> [(&'static str, bool, bool); 36] {
        [
            ("telegram", !self.telegram.is_empty(), true),
            ("discord", !self.discord.is_empty(), true),
            ("slack", !self.slack.is_empty(), true),
            ("mattermost", !self.mattermost.is_empty(), true),
            ("webhook", !self.webhook.is_empty(), true),
            ("imessage", !self.imessage.is_empty(), true),
            ("matrix", !self.matrix.is_empty(), true),
            ("signal", !self.signal.is_empty(), true),
            ("whatsapp", !self.whatsapp.is_empty(), true),
            ("linq", !self.linq.is_empty(), true),
            ("nextcloud_talk", !self.nextcloud_talk.is_empty(), true),
            ("email", !self.email.is_empty(), true),
            ("gmail_push", !self.gmail_push.is_empty(), true),
            ("irc", !self.irc.is_empty(), true),
            ("twitch", !self.twitch.is_empty(), true),
            ("lark", !self.lark.is_empty(), true),
            ("line", !self.line.is_empty(), true),
            ("dingtalk", !self.dingtalk.is_empty(), true),
            ("wecom", !self.wecom.is_empty(), true),
            ("wecom_ws", !self.wecom_ws.is_empty(), true),
            ("wechat", !self.wechat.is_empty(), true),
            ("qq", !self.qq.is_empty(), true),
            ("twitter", !self.twitter.is_empty(), true),
            ("mochat", !self.mochat.is_empty(), true),
            ("nostr", !self.nostr.is_empty(), true),
            ("clawdtalk", !self.clawdtalk.is_empty(), true),
            ("reddit", !self.reddit.is_empty(), true),
            ("bluesky", !self.bluesky.is_empty(), true),
            ("git", !self.git.is_empty(), true),
            ("voice_call", !self.voice_call.is_empty(), true),
            ("voice_wake", !self.voice_wake.is_empty(), false),
            ("voice_duplex", !self.voice_duplex.is_empty(), false),
            ("mqtt", !self.mqtt.is_empty(), false),
            ("amqp", !self.amqp.is_empty(), false),
            ("filesystem", !self.filesystem.is_empty(), false),
            ("plugin", !self.plugin.is_empty(), true),
        ]
    }

    /// Whether a channel named `name` (case-insensitive) is a known channel
    /// with at least one configured instance. Used by name-addressed surfaces
    /// (e.g. `heartbeat.target`) without hardcoding a subset of the registry.
    pub fn is_channel_configured(&self, name: &str) -> bool {
        let needle = name.to_ascii_lowercase();
        self.channel_presence()
            .iter()
            .any(|(canonical, configured, _)| *configured && *canonical == needle)
    }

    /// Whether `name` (case-insensitive) names a channel in the registry,
    /// regardless of whether it is configured.
    pub fn is_known_channel(&self, name: &str) -> bool {
        let needle = name.to_ascii_lowercase();
        self.channel_presence()
            .iter()
            .any(|(canonical, _, _)| *canonical == needle)
    }

    /// Whether `name` (case-insensitive) names a channel that can actually
    /// deliver an outbound message. Input-only transports (mqtt, amqp,
    /// voice_wake) are known and may be configured, but their `Channel::send`
    /// is a no-op, so they are not valid outbound targets.
    pub fn is_channel_deliverable(&self, name: &str) -> bool {
        let needle = name.to_ascii_lowercase();
        self.channel_presence()
            .iter()
            .any(|(canonical, _, deliverable)| *deliverable && *canonical == needle)
    }
}

fn default_channel_message_timeout_secs() -> u64 {
    300
}

impl Default for ChannelsConfig {
    fn default() -> Self {
        Self {
            cli: true,
            telegram: HashMap::new(),
            discord: HashMap::new(),
            slack: HashMap::new(),
            mattermost: HashMap::new(),
            webhook: HashMap::new(),
            imessage: HashMap::new(),
            matrix: HashMap::new(),
            signal: HashMap::new(),
            whatsapp: HashMap::new(),
            linq: HashMap::new(),
            nextcloud_talk: HashMap::new(),
            email: HashMap::new(),
            gmail_push: HashMap::new(),
            irc: HashMap::new(),
            twitch: HashMap::new(),
            lark: HashMap::new(),
            line: HashMap::new(),
            dingtalk: HashMap::new(),
            wecom: HashMap::new(),
            wecom_ws: HashMap::new(),
            wechat: HashMap::new(),
            qq: HashMap::new(),
            twitter: HashMap::new(),
            mochat: HashMap::new(),
            nostr: HashMap::new(),
            clawdtalk: HashMap::new(),
            reddit: HashMap::new(),
            bluesky: HashMap::new(),
            git: HashMap::new(),
            voice_call: HashMap::new(),
            voice_wake: HashMap::new(),
            voice_duplex: HashMap::new(),
            mqtt: HashMap::new(),
            amqp: HashMap::new(),
            filesystem: HashMap::new(),
            plugin: HashMap::new(),
            message_timeout_secs: default_channel_message_timeout_secs(),
            max_concurrent_per_channel: default_channel_max_concurrent_per_channel(),
            ack_reactions: true,
            show_tool_calls: false,
            session_persistence: true,
            session_backend: default_session_backend(),
            session_ttl_hours: 0,
            debounce_ms: 0,
        }
    }
}

/// Host-owned declaration of one installed channel-plugin binding.
///
/// This declaration is the complete operator-facing surface for a logical
/// channel instance: it names the installed package and whether the binding may
/// be admitted. It deliberately carries no plugin values — those stay in the
/// instance-keyed `plugins.entries` store so one plugin's secrets are never
/// reachable from another instance's declaration.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.plugin"]
pub struct PluginChannelConfig {
    /// Canonical package name from the installed plugin manifest.
    #[serde(default)]
    pub package: String,
    /// Whether this logical instance may be admitted at channel startup.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for PluginChannelConfig {
    fn default() -> Self {
        Self {
            package: String::new(),
            enabled: true,
        }
    }
}

/// Streaming mode for channels that support progressive message updates.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum StreamMode {
    /// No streaming -- send the complete response as a single message (default).
    #[default]
    Off,
    /// Update a draft message with every flush interval.
    Partial,
    /// Send the response as multiple separate messages at paragraph boundaries.
    #[serde(rename = "multi_message")]
    MultiMessage,
}

/// Matrix streaming mode for progressive response delivery.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum MatrixStreamMode {
    /// No streaming -- send the complete response as a single message (default).
    #[default]
    Off,
    /// Update a draft message with every flush interval.
    Partial,
    /// Stream progress into one sliding draft and send the final response as a separate message.
    #[serde(rename = "single_message")]
    SingleMessage,
    /// Send the response as multiple separate messages at paragraph boundaries.
    #[serde(rename = "multi_message")]
    MultiMessage,
}

/// Matrix single-message reasoning visibility.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum StreamReasoningMode {
    /// Do not emit reasoning-derived progress into the Matrix draft.
    Off,
    /// Emit liveness-only reasoning status ticks without raw reasoning text.
    #[default]
    Status,
    /// Emit raw provider reasoning text into the Matrix progress draft.
    Full,
}

/// Base policy for Matrix single-message tool argument progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum StreamToolArgumentBase {
    /// Do not display any arguments unless a per-tool rule includes them.
    None,
    /// Use OmnesAgent's conservative per-tool recommendations.
    #[default]
    Safe,
    /// Display every argument except runtime-internal fields, after leak scrubbing.
    All,
}

/// One entry in `MatrixConfig::stream_tool_arguments`.
///
/// A list may contain at most one defaults entry plus independent per-tool
/// rules. Entry order is not significant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
pub enum StreamToolArgumentEntry {
    /// Override inherited settings for tools without a matching rule.
    Defaults {
        default_base: StreamToolArgumentBase,
        /// Override the inherited per-value character cap. `0` disables it.
        #[serde(default)]
        argument_chars: Option<usize>,
    },
    /// Override or adjust inherited settings for one exact tool name.
    Tool {
        tool: String,
        #[serde(default)]
        base: Option<StreamToolArgumentBase>,
        #[serde(default)]
        include: Vec<String>,
        #[serde(default)]
        exclude: Vec<String>,
        /// Override the inherited per-value character cap. `0` disables it.
        #[serde(default)]
        argument_chars: Option<usize>,
    },
}

/// Where a channel registers its skill slash commands. `global` (default)
/// registers application-wide - the commands work everywhere the bot is, but
/// Discord takes up to ~1h to propagate changes. `guild` registers to each
/// guild in `guild_ids`, which propagates instantly (the right choice for fast
/// iteration and single-server bots). When `guild` is set with an empty
/// `guild_ids`, the channel logs a warning and falls back to global.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum SlashCommandScope {
    /// Application-wide commands (propagate in up to ~1h). Default.
    #[default]
    Global,
    /// Commands registered to each guild in `guild_ids` (instant propagation).
    Guild,
}

fn default_multi_message_delay_ms() -> u64 {
    800
}

pub const TELEGRAM_OFFICIAL_API_BASE_URL: &str = "https://api.telegram.org";

fn default_matrix_draft_update_interval_ms() -> u64 {
    1500
}

fn default_matrix_stream_draft_lines() -> usize {
    10
}

pub const DEFAULT_STREAM_TOOL_ARGUMENT_CHARS: usize = 60;
/// Smallest serialized Matrix message-content budget accepted by the
/// single-message delivery path. This leaves room for a non-empty body, the
/// JSON envelope, formatted Markdown, and an edit/reply relation.
pub const MATRIX_MIN_MESSAGE_MAX_BYTES: usize = 512;

fn default_matrix_message_max_bytes() -> usize {
    48_000
}

/// Telegram bot channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.telegram"]
pub struct TelegramConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Telegram Bot API token (from @BotFather). `#[serde(default)]` so a
    /// config that omits or later has it pruned (e.g. a freshly created
    /// alias with an empty token, stripped by `prune_empty_leaves` before
    /// write) still deserializes as an empty string - instead of failing
    /// with `missing field 'bot_token'` and getting dropped by the resilient
    /// salvage pass. `validate_bot_token` below still requires a real token
    /// once `enabled = true`.
    #[secret]
    #[tab(Connection)]
    #[serde(default)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub bot_token: String,
    /// Telegram Bot API base URL. Defaults to the official Telegram endpoint;
    /// set to a local Bot API server URL when self-hosting Telegram's bot API.
    #[tab(Connection)]
    #[serde(default = "default_telegram_api_base_url")]
    pub api_base_url: String,
    /// Streaming mode for progressive response delivery via message edits.
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_mode: StreamMode,
    /// Minimum interval (ms) between draft message edits to avoid rate limits.
    #[tab(Behavior)]
    #[serde(default = "default_draft_update_interval_ms")]
    pub draft_update_interval_ms: u64,
    /// Inbound message debounce window in milliseconds for this Telegram alias.
    /// When set, overrides the global `[channels].debounce_ms` for this channel
    /// only. `0` or unset falls back to the global value.
    #[tab(Behavior)]
    #[serde(default)]
    pub debounce_ms: Option<u64>,
    /// When true, a newer Telegram message from the same sender in the same chat
    /// cancels the in-flight request and starts a fresh response with preserved history.
    #[tab(Behavior)]
    #[serde(default)]
    pub interrupt_on_new_message: bool,
    /// When true, only respond to messages that @-mention the bot in groups.
    /// Direct messages are always processed.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,
    /// Override for the top-level `ack_reactions` setting. When `None`, the
    /// channel falls back to `[channels].ack_reactions`. When set
    /// explicitly, it takes precedence.
    #[tab(Behavior)]
    #[serde(default)]
    pub ack_reactions: Option<bool>,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// How long (seconds) to wait for the operator to tap an inline-keyboard
    /// button on a tool approval prompt before auto-denying. Default: 120.
    #[tab(Behavior)]
    #[serde(default = "default_telegram_approval_timeout_secs")]
    pub approval_timeout_secs: u64,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            api_base_url: default_telegram_api_base_url(),
            stream_mode: StreamMode::default(),
            draft_update_interval_ms: default_draft_update_interval_ms(),
            interrupt_on_new_message: false,
            mention_only: false,
            ack_reactions: None,
            proxy_url: None,
            approval_timeout_secs: default_telegram_approval_timeout_secs(),
            excluded_tools: Vec::new(),
            reply_min_interval_secs: 0,
            reply_queue_depth_max: 0,
            debounce_ms: None,
        }
    }
}

impl TelegramConfig {
    /// Validate this alias's bot-token placeholder and enabled-state rules.
    pub fn validate_bot_token(&self, alias: &str) -> Result<()> {
        validate_required_bot_token(
            &format!("channels.telegram.{alias}.bot_token"),
            self.enabled,
            &self.bot_token,
        )
    }
}

impl ChannelConfig for TelegramConfig {
    fn name() -> &'static str {
        "Telegram"
    }
    fn desc() -> &'static str {
        "connect your bot"
    }
}

/// Scope of inbound Discord reaction events the bot records.
///
/// Anything other than `Off` adds the GUILD_MESSAGE_REACTIONS and
/// DIRECT_MESSAGE_REACTIONS gateway intents (both unprivileged; no
/// Developer Portal toggle needed) and archives matching reactions to the
/// channel's `discord.db` sidecar when `archive` is enabled.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum DiscordReactionScope {
    /// Ignore inbound reaction events entirely (no reaction intents requested).
    #[default]
    Off,
    /// Record reactions to the bot's own messages only.
    Own,
    /// Record reactions to any message that passes the channel's filters.
    All,
}

/// Discord bot channel configuration.
///
/// `Default` is implemented below rather than derived. A derived `Default`
/// zeroes every field, which disagrees with the serde defaults, and for
/// `approval_timeout_secs` that disagreement is load-bearing: `0` is an
/// already-elapsed deadline, so an alias built in Rust would deny every
/// approval while an alias parsed from a file waits the documented 300s.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.discord"]
#[allow(clippy::struct_excessive_bools)]
pub struct DiscordConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Discord bot token (from Discord Developer Portal). `#[serde(default)]`
    /// for the same reason as `TelegramConfig::bot_token`: a missing token
    /// must deserialize as an empty string instead of salvage-dropping the
    /// alias; `validate_bot_token` still requires a real token once
    /// `enabled = true`.
    #[secret]
    #[tab(Connection)]
    #[serde(default)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub bot_token: String,
    /// Guild (server) IDs to restrict the bot to. Empty = listen across all
    /// guilds the bot is invited to. Migrated from the legacy `guild_id`
    /// singular field.
    #[tab(Advanced)]
    #[serde(default)]
    pub guild_ids: Vec<String>,
    /// Channel IDs to watch. Empty = watch every channel the bot can see.
    /// Used by the archive sidecar (when `archive = true`) and by the
    /// in-channel filter when set.
    #[tab(Advanced)]
    #[serde(default)]
    pub channel_ids: Vec<String>,
    /// When true, the channel opens a sidecar `discord.db` SQLite memory
    /// backend, archives every non-bot message it sees, and registers the
    /// `discord_search` tool against it. Default: false. Folded in from
    /// the legacy `[channels.discord-history]` block.
    #[tab(Advanced)]
    #[serde(default)]
    pub archive: bool,
    /// When true, process messages from other bots (not just humans).
    /// The bot still ignores its own messages to prevent feedback loops.
    #[tab(Advanced)]
    #[serde(default)]
    pub listen_to_bots: bool,
    /// When true, a newer Discord message from the same sender in the same channel
    /// cancels the in-flight request and starts a fresh response with preserved history.
    #[tab(Behavior)]
    #[serde(default)]
    pub interrupt_on_new_message: bool,
    /// When true, only respond to messages that @-mention the bot.
    /// Other messages in the guild are silently ignored.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,
    /// When true, register and serve Discord slash commands (e.g. `/ask`)
    /// over the Gateway WebSocket, in addition to message handling. Default
    /// false. (Prototype: currently registers a single `/ask <prompt>`.)
    #[tab(Behavior)]
    #[serde(default)]
    pub slash_commands: bool,
    /// Scope for registered slash commands: `global` (default, application-wide,
    /// ~1h propagation) or `guild` (registered to each `guild_ids` entry,
    /// instant). Only meaningful when `slash_commands = true`; `guild` with an
    /// empty `guild_ids` warns and falls back to global. Switching scope reaps
    /// owned commands from the now-inactive scope; note that *removing* a guild
    /// from `guild_ids` (without switching scope) does not reap that guild's
    /// commands - remove the bot from the guild, or switch scope, to clear them.
    #[tab(Behavior)]
    #[serde(default)]
    pub slash_command_scope: SlashCommandScope,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Streaming mode for progressive response delivery.
    /// `off` (default): single message. `partial`: editable draft updates.
    /// `multi_message`: split response into separate messages at paragraph boundaries.
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_mode: StreamMode,
    /// Minimum interval (ms) between draft message edits to avoid rate limits.
    /// Only used when `stream_mode = "partial"`.
    #[tab(Behavior)]
    #[serde(default = "default_draft_update_interval_ms")]
    pub draft_update_interval_ms: u64,
    /// Delay (ms) between sending each message chunk in multi-message mode.
    /// Only used when `stream_mode = "multi_message"`.
    #[tab(Behavior)]
    #[serde(default = "default_multi_message_delay_ms")]
    pub multi_message_delay_ms: u64,
    /// Stall-watchdog timeout in seconds. When non-zero, the bot will abort
    /// and retry if no progress is made within this duration. 0 = disabled.
    #[tab(Advanced)]
    #[serde(default)]
    pub stall_timeout_secs: u64,
    /// Raw gateway intent mask override. When set, this exact value is sent
    /// in IDENTIFY instead of the derived mask: an operator escape hatch
    /// for downstream deployments that consume gateway events through
    /// custom builds or forks. The operator owns the consequences:
    /// privileged bits still need their Developer Portal toggles, and
    /// dropping baseline bits silences message handling (a warning is
    /// logged). Unset (default) = derive the mask from the channel's
    /// feature config.
    #[tab(Advanced)]
    #[serde(default)]
    pub intents_mask: Option<u64>,
    /// Which inbound reactions to record: `off` (default: reaction events
    /// are not even requested from the gateway), `own` (reactions to the
    /// bot's messages), or `all` (reactions to any message passing the
    /// channel's filters). Recorded reactions are archived to `discord.db`
    /// when `archive` is enabled, and removed again when a user removes
    /// their reaction (bulk clears, remove-all / remove-emoji, are not
    /// yet swept). A set `intents_mask` overrides the derived mask: if it
    /// omits the reaction intents, nothing is recorded.
    #[tab(Behavior)]
    #[serde(default)]
    pub reaction_notifications: DiscordReactionScope,
    /// Seconds to wait for operator approval on `always_ask` tools before auto-denying.
    #[tab(Behavior)]
    #[serde(default = "default_channel_approval_timeout_secs")]
    pub approval_timeout_secs: u64,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

impl Default for DiscordConfig {
    /// Built by deserializing an empty object, so the serde defaults are the
    /// only source of truth and the two cannot disagree. Every field here
    /// either declares a serde default or is an `Option`, which is what
    /// makes the empty object total.
    fn default() -> Self {
        serde_json::from_str("{}")
            .expect("every DiscordConfig field declares a serde default or is Option")
    }
}

impl DiscordConfig {
    /// Validate this alias's bot-token placeholder and enabled-state rules.
    /// Mirrors `TelegramConfig::validate_bot_token`.
    pub fn validate_bot_token(&self, alias: &str) -> Result<()> {
        validate_required_bot_token(
            &format!("channels.discord.{alias}.bot_token"),
            self.enabled,
            &self.bot_token,
        )
    }
}

impl ChannelConfig for DiscordConfig {
    fn name() -> &'static str {
        "Discord"
    }
    fn desc() -> &'static str {
        "connect your bot"
    }
}

/// Default number of prior Slack thread messages included on first encounter.
pub const DEFAULT_SLACK_THREAD_CONTEXT_MAX_MESSAGES: usize = 0;
/// Slack's `conversations.replies` request limit used by thread hydration.
pub const MAX_SLACK_THREAD_CONTEXT_MAX_MESSAGES: usize = 50;

/// Slack bot channel configuration.
///
/// `Default` is implemented below rather than derived. A derived `Default`
/// zeroes every field, which disagrees with the serde defaults, and for
/// `approval_timeout_secs` that disagreement is load-bearing: `0` is an
/// already-elapsed deadline, so an alias built in Rust would deny every
/// approval while an alias parsed from a file waits the documented 300s.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.slack"]
#[allow(clippy::struct_excessive_bools)]
pub struct SlackConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Slack bot OAuth token (xoxb-...). Optional in config: when unset or
    /// empty it is resolved at channel construction from
    /// `OMNESAGENT_SLACK_BOT_TOKEN`, then `SLACK_BOT_TOKEN`. `#[serde(default)]`
    /// so a config that omits it still deserializes - the env fallback then
    /// supplies it - instead of failing with `missing field 'bot_token'`.
    ///
    #[secret]
    #[serde(default)]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub bot_token: Option<String>,
    /// Slack app-level token for Socket Mode (xapp-...). When unset or empty,
    /// resolved at channel construction from `OMNESAGENT_SLACK_APP_TOKEN`, then
    /// `SLACK_APP_TOKEN`. `#[serde(default)]` makes omission explicit (an
    /// `Option` field is already omittable, but this keeps it consistent with
    /// `bot_token`).
    #[secret]
    #[serde(default)]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub app_token: Option<String>,
    /// Explicit list of channel IDs to watch.
    /// Empty = listen across all accessible channels.
    /// Migrated from the legacy `channel_id` singular field.
    #[tab(Advanced)]
    #[serde(default)]
    pub channel_ids: Vec<String>,
    /// When true, a newer Slack message from the same sender in the same channel
    /// cancels the in-flight request and starts a fresh response with preserved history.
    #[tab(Behavior)]
    #[serde(default)]
    pub interrupt_on_new_message: bool,
    /// When true (default), replies stay in the originating Slack thread.
    /// When false, replies go to the channel root instead.
    #[tab(Advanced)]
    #[serde(default)]
    pub thread_replies: Option<bool>,
    /// When true, only respond to messages that @-mention the bot in groups.
    /// Direct messages remain allowed.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,
    /// When true (and `mention_only` is also true), messages inside a Slack
    /// thread must also @-mention the bot to trigger a response. By default,
    /// thread replies are allowed through without a mention so the bot can
    /// keep a back-and-forth going without the user repeating @-mentions.
    /// Set this to true in channels shared with human discussion where the
    /// bot should stay silent unless explicitly addressed.
    #[tab(Advanced)]
    #[serde(default)]
    pub strict_mention_in_thread: bool,
    /// Prior thread messages to hydrate on the first bot interaction. `0`
    /// disables hydration. Maximum: 50.
    #[tab(Advanced)]
    #[serde(default)]
    pub thread_context_max_messages: Option<usize>,
    /// Use the newer Slack `markdown` block type (12 000 char limit, richer formatting).
    /// Defaults to false (uses universally supported `section` blocks with `mrkdwn`).
    /// Enable this only if your Slack workspace supports the `markdown` block type.
    #[tab(Advanced)]
    #[serde(default)]
    pub use_markdown_blocks: bool,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Enable progressive draft message streaming via `chat.update`.
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_drafts: bool,
    /// Minimum interval (ms) between draft message edits to avoid Slack rate limits.
    #[tab(Behavior)]
    #[serde(default = "default_slack_draft_update_interval_ms")]
    pub draft_update_interval_ms: u64,
    /// Emoji reaction name (without colons) that cancels an in-flight request.
    /// For example, `"x"` means reacting with `:x:` cancels the task.
    /// Leave unset to disable reaction-based cancellation.
    #[tab(Advanced)]
    #[serde(default)]
    pub cancel_reaction: Option<String>,
    /// Seconds to wait for operator approval on `always_ask` tools before auto-denying.
    #[tab(Behavior)]
    #[serde(default = "default_channel_approval_timeout_secs")]
    pub approval_timeout_secs: u64,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

fn default_slack_draft_update_interval_ms() -> u64 {
    1200
}

impl Default for SlackConfig {
    /// Built by deserializing an empty object, so the serde defaults are the
    /// only source of truth and the two cannot disagree. Every field here
    /// either declares a serde default or is an `Option`, which is what
    /// makes the empty object total.
    fn default() -> Self {
        serde_json::from_str("{}")
            .expect("every SlackConfig field declares a serde default or is Option")
    }
}

impl ChannelConfig for SlackConfig {
    fn name() -> &'static str {
        "Slack"
    }
    fn desc() -> &'static str {
        "connect your bot"
    }
}

impl SlackConfig {
    /// Resolve the configured thread-context depth without copying config
    /// state into the channel runtime.
    pub fn effective_thread_context_max_messages(&self) -> usize {
        self.thread_context_max_messages
            .unwrap_or(DEFAULT_SLACK_THREAD_CONTEXT_MAX_MESSAGES)
    }

    /// Resolve the effective Slack bot token: the configured `bot_token` when
    /// set and non-empty, else the `OMNESAGENT_SLACK_BOT_TOKEN` env var, else
    /// `SLACK_BOT_TOKEN`. Returns `None` when none is available. Resolving here
    /// (rather than writing the env value back into the config struct) keeps an
    /// env-supplied secret out of any config that might later be persisted to
    /// disk.
    pub fn resolved_bot_token(&self) -> Option<String> {
        resolve_slack_token(self.bot_token.as_deref(), "BOT")
    }

    /// Resolve the effective Slack app-level (Socket Mode) token: configured
    /// `app_token` when set and non-empty, else `OMNESAGENT_SLACK_APP_TOKEN`,
    /// else `SLACK_APP_TOKEN`. Returns `None` when none is available.
    pub fn resolved_app_token(&self) -> Option<String> {
        resolve_slack_token(self.app_token.as_deref(), "APP")
    }
}

/// Shared token resolution for [`SlackConfig`]: a non-empty configured value
/// wins; otherwise the `OMNESAGENT_SLACK_<KIND>_TOKEN` env var takes precedence
/// over the conventional `SLACK_<KIND>_TOKEN`. Blank values (config or env) are
/// treated as absent. `kind` is `"BOT"` or `"APP"`.
fn resolve_slack_token(configured: Option<&str>, kind: &str) -> Option<String> {
    if let Some(value) = configured {
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    for var in [
        format!("OMNESAGENT_SLACK_{kind}_TOKEN"),
        format!("SLACK_{kind}_TOKEN"),
    ] {
        if let Ok(value) = std::env::var(&var) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// How the Mattermost channel receives inbound messages.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum MattermostListenMode {
    /// Poll REST API every 3 seconds. The default — all existing configs
    /// continue to work without changes.
    #[default]
    Polling,
    /// Connect to `/api/v4/websocket` for near-real-time event delivery.
    Websocket,
}

/// Mattermost bot channel configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.mattermost"]
pub struct MattermostConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Mattermost server URL (e.g. `"https://mattermost.example.com"`).
    #[tab(Connection)]
    pub url: String,
    /// Mattermost bot access token. When unset, the channel falls back to
    /// the login flow using `login_id` + `password`.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default)]
    pub bot_token: Option<String>,
    /// Login ID (email or username) for the password login flow. Used only
    /// when `bot_token` is unset; both `login_id` and `password` must be
    /// set together.
    #[tab(Connection)]
    #[serde(default)]
    pub login_id: Option<String>,
    /// Account password for the login flow. Used only when `bot_token` is
    /// unset; both `login_id` and `password` must be set together.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default)]
    pub password: Option<String>,
    /// Channel IDs to restrict the bot to. Empty or `["*"]` = auto-discover
    /// every channel the bot can read (public, private, DMs, group DMs) and
    /// poll them all. Explicit IDs disable discovery and pin the bot to the
    /// listed channels only. Migrated from the legacy `channel_id` singular
    /// field.
    #[tab(Advanced)]
    #[serde(default)]
    pub channel_ids: Vec<String>,
    /// Team IDs to restrict auto-discovery to. Empty = discover across every
    /// team the bot belongs to. Non-empty = only discover public/private
    /// channels whose `team_id` is in this list. DMs and group DMs (which
    /// have no team) are governed by `discover_dms` instead.
    #[tab(Advanced)]
    #[serde(default)]
    pub team_ids: Vec<String>,
    /// When true (default), auto-discovery includes DM (`type=D`) and group
    /// DM (`type=G`) channels. Set false to restrict the bot to public and
    /// private team channels only. Has no effect when `channel_ids` lists
    /// explicit IDs. Defaults to `true` at the call site via
    /// `discover_dms.unwrap_or(true)`.
    #[tab(Advanced)]
    #[serde(default)]
    pub discover_dms: Option<bool>,
    /// When true (default), replies thread on the original post.
    /// When false, replies go to the channel root.
    #[tab(Advanced)]
    #[serde(default)]
    pub thread_replies: Option<bool>,
    /// When true, only respond to messages that @-mention the bot. Other
    /// messages in the channel are silently ignored. DM and group DM
    /// channels always bypass this filter: a 1:1 (or small-group) direct
    /// conversation has no ambient noise to gate against, so every message
    /// is treated as addressed to the bot.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: Option<bool>,
    /// When true, a newer Mattermost message from the same sender in the same channel
    /// cancels the in-flight request and starts a fresh response with preserved history.
    #[tab(Behavior)]
    #[serde(default)]
    pub interrupt_on_new_message: bool,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// Listen mode: `"polling"` (REST API every 3s, default) or `"websocket"`
    /// (persistent WebSocket connection to `/api/v4/websocket` for near-real-time
    /// event delivery). WebSocket mode reduces server load and delivers events
    /// faster but requires a WebSocket-capable Mattermost server (v4.0+).
    #[serde(default)]
    pub listen_mode: MattermostListenMode,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

impl ChannelConfig for MattermostConfig {
    fn name() -> &'static str {
        "Mattermost"
    }
    fn desc() -> &'static str {
        "connect to your bot"
    }
}

/// Webhook channel configuration.
///
/// Receives messages via HTTP POST and sends replies to a configurable outbound URL.
/// This is the "universal adapter" for any system that supports webhooks.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.webhook"]
pub struct WebhookConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Port to listen on for incoming webhooks.
    #[tab(Advanced)]
    #[serde(default = "default_webhook_channel_port")]
    pub port: u16,
    /// URL path to listen on (default: `/webhook`).
    #[tab(Advanced)]
    #[serde(default)]
    pub listen_path: Option<String>,
    /// URL to POST/PUT outbound messages to.
    #[tab(Advanced)]
    #[serde(default)]
    pub send_url: Option<String>,
    /// HTTP method for outbound messages (`POST` or `PUT`). Default: `POST`.
    #[tab(Advanced)]
    #[serde(default)]
    pub send_method: Option<String>,
    /// Optional `Authorization` header value for outbound requests.
    #[tab(Connection)]
    #[serde(default)]
    #[secret]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub auth_header: Option<String>,
    /// Shared secret for webhook signature verification (HMAC-SHA256).
    /// The channel will refuse to start without one. Set
    /// `[channels.webhook.<alias>].secret` in config.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub secret: Option<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,

    /// Maximum number of retry attempts for outbound sends on transient failures
    /// (network errors, 429, 5xx). Set to `0` to disable retries. Default: `3`.
    #[serde(default)]
    pub max_retries: Option<u32>,
    /// Base delay in milliseconds for exponential backoff between retries. Default: `500`.
    /// Values below `1` are clamped to `1ms` at runtime to avoid busy-retry loops.
    #[serde(default)]
    pub retry_base_delay_ms: Option<u64>,
    /// Maximum delay cap in milliseconds for any single retry wait. Default: `30000` (30s).
    /// Values below `1` are clamped to `1ms` at runtime to avoid busy-retry loops.
    #[serde(default)]
    pub retry_max_delay_ms: Option<u64>,
}

pub const DEFAULT_WEBHOOK_CHANNEL_PORT: u16 = 8090;

fn default_webhook_channel_port() -> u16 {
    DEFAULT_WEBHOOK_CHANNEL_PORT
}

impl ChannelConfig for WebhookConfig {
    fn name() -> &'static str {
        "Webhook"
    }
    fn desc() -> &'static str {
        "HTTP endpoint"
    }
}

/// iMessage channel configuration (macOS only).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.imessage"]
pub struct IMessageConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

impl ChannelConfig for IMessageConfig {
    fn name() -> &'static str {
        "iMessage"
    }
    fn desc() -> &'static str {
        "macOS only"
    }
}

/// Matrix channel configuration.
///
/// `Default` is implemented below rather than derived. A derived `Default`
/// zeroes every field, which disagrees with the serde defaults, and for
/// `approval_timeout_secs` that disagreement is load-bearing: `0` is an
/// already-elapsed deadline, so an alias built in Rust would deny every
/// approval while an alias parsed from a file waits the documented 300s.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.matrix"]
pub struct MatrixConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Matrix server name or homeserver URL (e.g. `"matrix.org"` or
    /// `"https://matrix.example.org"`). Server names use standard
    /// `/.well-known/matrix/client` discovery.
    #[tab(Connection)]
    pub homeserver: String,
    /// Matrix access token for the bot account. When unset, the channel
    /// falls back to password login using `user_id` + `password`.
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default)]
    pub access_token: Option<String>,
    /// Optional Matrix user ID (e.g. `"@bot:matrix.org"`).
    #[tab(Connection)]
    #[serde(default)]
    pub user_id: Option<String>,
    /// Optional Matrix device ID.
    #[tab(Connection)]
    #[serde(default)]
    pub device_id: Option<String>,
    /// Allowed Matrix room IDs. Empty = allow all rooms the bot has joined.
    /// Entries are matched literally against the canonical room ID
    /// (`!abc:server`) of each incoming message; `#room:server` aliases are
    /// not resolved for this allowlist (they are resolved only for outbound
    /// delivery targets such as cron `delivery.to`).
    #[tab(Behavior)]
    #[serde(default)]
    pub allowed_rooms: Vec<String>,
    /// Whether to interrupt an in-flight agent response when a new message arrives.
    #[tab(Behavior)]
    #[serde(default)]
    pub interrupt_on_new_message: bool,
    /// Streaming mode for progressive response delivery.
    /// `"off"` (default): single final message. `"partial"`: edit-in-place draft.
    /// `"single_message"`: progress draft plus separate final message.
    /// `"multi_message"`: paragraph-split delivery.
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_mode: MatrixStreamMode,
    /// Minimum interval (ms) between Matrix draft edits in Partial mode and
    /// thinking/reasoning progress edits in SingleMessage mode.
    #[tab(Behavior)]
    #[serde(default = "default_matrix_draft_update_interval_ms")]
    pub draft_update_interval_ms: u64,
    /// Delay (ms) between sending each paragraph in MultiMessage mode.
    #[tab(Behavior)]
    #[serde(default = "default_multi_message_delay_ms")]
    pub multi_message_delay_ms: u64,
    /// Maximum number of progress lines kept in the Matrix single-message
    /// streaming draft. Set to 0 to remove the line-count limit; all lines
    /// still compete within one byte-bounded Matrix draft event.
    #[tab(Behavior)]
    #[serde(default = "default_matrix_stream_draft_lines")]
    pub stream_draft_lines: usize,
    /// Serialized Matrix event-content byte budget for single-message
    /// streaming draft edits and the separate final response. The rendered
    /// Markdown body, generated HTML, and reply/edit relation all count
    /// toward this limit. Oversized progress drops complete oldest entries;
    /// the separate final response retains a UTF-8-safe prefix. Values below
    /// 512 use that minimum, which leaves room for a non-empty serialized event.
    #[tab(Behavior)]
    #[serde(default = "default_matrix_message_max_bytes")]
    pub message_max_bytes: usize,
    /// Delete the Matrix single-message progress draft before sending the final
    /// response. When false, durable progress remains as a visible transcript;
    /// placeholder-only drafts are still removed before the final response.
    #[tab(Behavior)]
    #[serde(default = "default_true")]
    pub stream_draft_delete: bool,
    /// Matrix single-message reasoning visibility. `"off"` suppresses
    /// reasoning-derived draft updates, `"status"` emits liveness ticks without
    /// raw reasoning text, and `"full"` emits raw provider reasoning text into
    /// the progress draft.
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_reasoning: StreamReasoningMode,
    /// Tool arguments shown in Matrix single-message progress lines. Missing or
    /// empty means the conservative `safe` defaults. Use one
    /// `{ default_base = "none" | "safe" | "all" }` entry for inherited
    /// settings, then exact-name tool entries with optional `base`, `include`,
    /// `exclude`, and `argument_chars` adjustments. `argument_chars` limits
    /// each displayed value and defaults to 60; `0` disables that limit.
    /// Unknown tools resolve to no arguments under `safe`; every selected value
    /// is leak-scrubbed before display.
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_tool_arguments: Vec<StreamToolArgumentEntry>,
    /// When true, only respond to messages that @-mention the bot in groups.
    /// Direct messages are always processed.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,
    /// Optional Matrix recovery key for automatic E2EE key backup restore.
    /// When set, OmnesAgent recovers room keys and cross-signing secrets on startup.
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default)]
    pub recovery_key: Option<String>,
    /// Optional login password for Matrix account (used for initial login flow).
    #[secret]
    #[credential_class = "encrypted_secret"]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    #[serde(default)]
    pub password: Option<String>,
    /// Seconds to wait for operator approval on `always_ask` tools before auto-denying.
    #[tab(Behavior)]
    #[serde(default = "default_channel_approval_timeout_secs")]
    pub approval_timeout_secs: u64,
    /// When true (default), replies are sent as thread replies. Starts a new thread from the
    /// incoming message when none exists. When false, only continues existing threads.
    #[tab(Behavior)]
    #[serde(default = "default_true")]
    pub reply_in_thread: bool,
    /// Override for the top-level `[channels].ack_reactions`. When
    /// `None`, falls back to the channels-wide default. When set
    /// explicitly (`true`/`false`), takes precedence for this Matrix
    /// instance only.
    #[tab(Behavior)]
    #[serde(default)]
    pub ack_reactions: Option<bool>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

impl Default for MatrixConfig {
    /// Built by deserializing an object supplying only `homeserver`, the one
    /// field with no serde default - a Matrix channel must always name a
    /// homeserver, so it stays required rather than gaining one. Every other
    /// field either declares a serde default or is an `Option`, so this is
    /// the minimal object that makes the deserialize total; those defaults
    /// stay the only source of truth for everything but `homeserver`.
    fn default() -> Self {
        serde_json::from_str(r#"{"homeserver":""}"#)
            .expect("every other MatrixConfig field declares a serde default or is Option")
    }
}

fn validate_stream_tool_argument_names<'a>(
    entry_path: &str,
    field_name: &str,
    fields: &'a [String],
) -> Result<std::collections::HashSet<&'a str>> {
    let mut names = std::collections::HashSet::new();
    for (field_index, field) in fields.iter().enumerate() {
        if field.is_empty() || field.trim() != field {
            anyhow::bail!(
                "{entry_path}.{field_name}[{field_index}] must be a non-empty exact argument name without surrounding whitespace"
            );
        }
        if !names.insert(field.as_str()) {
            anyhow::bail!("{entry_path}.{field_name} contains duplicate argument '{field}'");
        }
    }
    Ok(names)
}

impl MatrixConfig {
    /// Effective UTF-8 body budget for Matrix single-message streaming.
    ///
    /// A serialized Matrix event has fixed JSON, message, and relation
    /// overhead in addition to UTF-8 source text. Keeping this floor avoids
    /// accepting a budget that cannot contain even a one-scalar event.
    pub fn effective_message_max_bytes(&self) -> usize {
        self.message_max_bytes.max(MATRIX_MIN_MESSAGE_MAX_BYTES)
    }

    /// Validate the order-independent tool argument display policy.
    pub fn validate_stream_tool_arguments(&self) -> Result<()> {
        let mut saw_defaults = false;
        let mut tools = std::collections::HashSet::new();

        for (index, entry) in self.stream_tool_arguments.iter().enumerate() {
            let entry_path = format!("stream_tool_arguments[{index}]");
            match entry {
                StreamToolArgumentEntry::Defaults { .. } => {
                    if saw_defaults {
                        anyhow::bail!(
                            "{entry_path}.default_base duplicates the list's default_base entry"
                        );
                    }
                    saw_defaults = true;
                }
                StreamToolArgumentEntry::Tool {
                    tool,
                    include,
                    exclude,
                    ..
                } => {
                    if tool.is_empty() || tool.trim() != tool {
                        anyhow::bail!(
                            "{entry_path}.tool must be a non-empty exact tool name without surrounding whitespace"
                        );
                    }
                    if !tools.insert(tool.as_str()) {
                        anyhow::bail!("{entry_path}.tool duplicates the rule for tool '{tool}'");
                    }

                    let included =
                        validate_stream_tool_argument_names(&entry_path, "include", include)?;
                    let excluded =
                        validate_stream_tool_argument_names(&entry_path, "exclude", exclude)?;
                    for field in &excluded {
                        if included.contains(*field) {
                            anyhow::bail!(
                                "{entry_path} includes and excludes the same argument '{field}'"
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl ChannelConfig for MatrixConfig {
    fn name() -> &'static str {
        "Matrix"
    }
    fn desc() -> &'static str {
        "self-hosted chat"
    }
}

/// Signal channel configuration.
///
/// `Default` is implemented below rather than derived. A derived `Default`
/// zeroes every field, which disagrees with the serde defaults, and for
/// `approval_timeout_secs` that disagreement is load-bearing: `0` is an
/// already-elapsed deadline, so an alias built in Rust would deny every
/// approval while an alias parsed from a file waits the documented 300s.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.signal"]
pub struct SignalConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Base URL for the signal-cli HTTP daemon (e.g. `"http://127.0.0.1:8686"`).
    #[tab(Connection)]
    pub http_url: String,
    /// E.164 phone number of the signal-cli account (e.g. "+1234567890").
    #[tab(Connection)]
    pub account: String,
    /// Group IDs to filter messages. Empty = accept all messages (DMs and
    /// groups). When non-empty, only messages from listed groups are
    /// accepted (DMs are still accepted unless `dm_only` flips the policy
    /// to DMs-only). Migrated from the legacy `group_id` singular field.
    #[tab(Advanced)]
    #[serde(default)]
    pub group_ids: Vec<String>,
    /// When true, only accept direct messages and ignore all group traffic.
    /// Mutually exclusive with `group_ids` (which is ignored when this is
    /// set). Migrated from the legacy `group_id = "dm"` sentinel.
    #[tab(Advanced)]
    #[serde(default)]
    pub dm_only: bool,
    /// Skip messages that are attachment-only (no text body).
    #[tab(Advanced)]
    #[serde(default)]
    pub ignore_attachments: bool,
    /// Skip incoming story messages.
    #[tab(Advanced)]
    #[serde(default)]
    pub ignore_stories: bool,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Seconds to wait for operator approval on `always_ask` tools before auto-denying.
    #[tab(Behavior)]
    #[serde(default = "default_channel_approval_timeout_secs")]
    pub approval_timeout_secs: u64,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

impl Default for SignalConfig {
    /// Built by deserializing an object supplying only `http_url` and
    /// `account`, the two fields with no serde default - a Signal channel
    /// must always name its daemon and account, so they stay required
    /// rather than gaining one. Every other field either declares a serde
    /// default or is an `Option`, so this is the minimal object that makes
    /// the deserialize total; those defaults stay the only source of truth
    /// for everything but `http_url` and `account`.
    fn default() -> Self {
        serde_json::from_str(r#"{"http_url":"","account":""}"#)
            .expect("every other SignalConfig field declares a serde default or is Option")
    }
}

impl SignalConfig {
    /// Whether both required credentials (`http_url`, `account`) are
    /// present. Mirrors `WhatsAppConfig::is_cloud_config`'s role: the
    /// channel orchestrator uses this bool to decide whether to build the
    /// channel at all, skipping (with a warning) an enabled-but-uncredentialed
    /// alias instead of building a listener that can never connect and
    /// crashloops its per-channel supervisor.
    pub fn has_required_credentials(&self) -> bool {
        !crate::traits::is_unset_display_value(&self.http_url)
            && !crate::traits::is_unset_display_value(&self.account)
    }

    /// Validate this alias's required credentials once enabled. Mirrors
    /// `TelegramConfig::validate_bot_token` / `DiscordConfig::validate_bot_token`:
    /// an enabled Signal channel built with an empty `http_url` or `account`
    /// never connects and crashloops its per-channel supervisor, so reject
    /// it at config-load time instead.
    pub fn validate_required(&self, alias: &str) -> Result<()> {
        let enabled_path = format!("channels.signal.{alias}.enabled");
        validate_required_field(
            &format!("channels.signal.{alias}.http_url"),
            &enabled_path,
            self.enabled,
            &self.http_url,
        )?;
        validate_required_field(
            &format!("channels.signal.{alias}.account"),
            &enabled_path,
            self.enabled,
            &self.account,
        )
    }
}

impl ChannelConfig for SignalConfig {
    fn name() -> &'static str {
        "Signal"
    }
    fn desc() -> &'static str {
        "An open-source, encrypted messaging service"
    }
}

/// WhatsApp Web usage mode.
///
/// The mode no longer decides WHETHER the chat policies apply. `dm_policy` and
/// `group_policy` are consulted under BOTH modes, and an unrecognized sender is
/// dropped before the message is logged or dispatched either way.
///
/// `Personal` additionally applies the self-chat exception, so `self_chat_mode`
/// is consulted only there.
/// `Business` (default) is the same admission path without that exception.
///
/// Neither mode consults `allowed_numbers`. That is a V2 field which migrates
/// into `peer_groups` on load, so it names no knob the current model exposes;
/// senders resolve from `[peer_groups.<name>].external_peers` scoped to the
/// alias.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, omnesagent_macros::ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WhatsAppWebMode {
    /// Respond to messages passing `dm_policy` and `group_policy` (default).
    #[default]
    Business,
    /// As business mode, and additionally applies the self-chat semantics
    /// (`self_chat_mode` and the fromMe handling). Both modes run the same
    /// linked-device session, and WhatsApp can mirror an operator's own
    /// messages as `fromMe` under either, so that scoping is a property of
    /// this branch rather than of the account.
    Personal,
}

/// Policy for a particular WhatsApp chat type (DMs or groups).
///
/// Applied under both `mode = "business"` and `mode = "personal"`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, omnesagent_macros::ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WhatsAppChatPolicy {
    /// Only respond to recognized senders (default). Senders are resolved from
    /// the channel's peer group, via `[peer_groups.<name>].external_peers`.
    #[default]
    Allowlist,
    /// Ignore all messages in this chat type.
    Ignore,
    /// Respond to every message regardless of allowlist.
    All,
}

/// Whether an empty `allowed_groups` is a capability the operator is LOSING.
///
/// The change this reports is narrow: the empty group-identity gate moves from
/// admit-all to policy-selected. Only a configuration that previously reached
/// the chat-type gate and was admitted there loses anything.
///
/// Two policies lose nothing and are therefore never reported:
///
/// - `all` admits every group before and after.
/// - `ignore` rejects every group before and after. It is mode-independent in
///   the chat-type gate, so a Business channel with `ignore` was already closed
///   just as a Personal one was. Reporting it would tell an operator their
///   channel used to answer every group and offer ways to reopen a policy they
///   set deliberately.
///
/// `mode` is accepted so callers need not know that the answer is currently
/// mode-independent, and so a future policy that IS mode-sensitive has a place
/// to land without changing every call site.
///
/// Shared rather than duplicated: the Web transport emits a startup notice from
/// this same predicate, so the runtime behavior and the `config validate` warning
/// cannot drift into disagreeing about which configurations changed.
pub fn whatsapp_empty_group_list_is_newly_closed(
    _mode: &WhatsAppWebMode,
    group_policy: &WhatsAppChatPolicy,
) -> bool {
    !matches!(
        group_policy,
        WhatsAppChatPolicy::All | WhatsAppChatPolicy::Ignore
    )
}

/// How to restore group access, phrased for the policy actually in force.
///
/// Shared rather than duplicated for the same reason as the predicate above:
/// the Web transport's startup notice and the `config validate` warning must
/// not offer different remedies for the same configuration.
///
/// The `ignore` arm exists because naming `allowed_groups` there would be an
/// INEFFECTIVE remedy. A listed group passes the group-identity gate and is
/// then dropped by the chat-type gate, so populating the list changes nothing
/// and the operator has to choose a different policy instead.
pub fn whatsapp_empty_group_list_remedy(group_policy: &WhatsAppChatPolicy) -> &'static str {
    match group_policy {
        WhatsAppChatPolicy::Ignore => {
            "group_policy = \"ignore\" serves no group by design; set \
             group_policy = \"all\" to admit every group, or \
             group_policy = \"allowlist\" together with the group JIDs you \
             intend to serve"
        }
        _ => {
            "list the group JIDs you intend to serve in allowed_groups, or \
             set group_policy = \"all\" to admit every group"
        }
    }
}

/// WhatsApp channel configuration (Cloud API or Web mode).
///
/// Set `phone_number_id` for Cloud API mode, or `session_path` for Web mode.
///
/// `Default` is implemented below rather than derived. A derived `Default`
/// zeroes every field, which disagrees with the serde defaults, and for
/// `approval_timeout_secs` that disagreement is load-bearing: `0` is an
/// already-elapsed deadline, so an alias built in Rust would deny every
/// approval while an alias parsed from a file waits the documented 300s.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.whatsapp"]
pub struct WhatsAppConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Access token from Meta Business Suite (Cloud API mode)
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub access_token: Option<String>,
    /// Phone number ID from Meta Business API (Cloud API mode)
    #[tab(Connection)]
    #[serde(default)]
    pub phone_number_id: Option<String>,
    /// Webhook verify token (you define this, Meta sends it back for verification)
    /// Only used in Cloud API mode
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub verify_token: Option<String>,
    /// App secret from Meta Business Suite (for webhook signature verification)
    /// Can also be set with the alias-qualified generic environment override:
    /// `OMNESAGENT_channels__whatsapp__<alias>__app_secret`.
    /// Only used in Cloud API mode.
    ///
    /// Required to receive webhooks. Inbound requests are signature-verified,
    /// and with no secret configured the gateway cannot verify them, so it
    /// refuses them with `401` rather than accepting them unverified.
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub app_secret: Option<String>,
    /// Session database path for WhatsApp Web client (Web mode)
    /// When set, enables native WhatsApp Web mode with wa-rs
    #[tab(Connection)]
    #[serde(default)]
    pub session_path: Option<String>,
    /// Phone number for pair code linking (Web mode, optional)
    /// Format: country code + number (e.g., "15551234567")
    /// If not set, QR code pairing will be used
    #[tab(Connection)]
    #[serde(default)]
    pub pair_phone: Option<String>,
    /// Custom pair code for linking (Web mode, optional)
    /// Leave empty to let WhatsApp generate one
    #[tab(Connection)]
    #[serde(default)]
    pub pair_code: Option<String>,
    /// Override the WhatsApp Web WebSocket URL (Web mode, optional). Used
    /// by integration tests and proxy setups; leave unset to use the
    /// default endpoint that ships with `wa-rs`.
    #[tab(Connection)]
    #[serde(default)]
    pub ws_url: Option<String>,
    /// Display name announced to contacts (Web mode, optional)
    /// Applied on connect when it differs from the name the linked device
    /// already carries; leave unset to keep the account's own name
    #[tab(Connection)]
    #[serde(default)]
    pub push_name: Option<String>,
    /// When true, only respond to messages that @-mention the bot in groups (Web mode only).
    /// Direct messages are always processed.
    /// Bot identity is resolved from the wa-rs device at runtime; `pair_phone` seeds it on first connect.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,
    /// When true in WhatsApp Web group chats, unaddressed messages that pass
    /// sender/chat authorization are recorded as passive conversation context
    /// without starting an agent turn. Default: `false`.
    #[tab(Behavior)]
    #[serde(default)]
    pub passive_group_context: bool,
    /// Cancel an in-flight response from this channel sender when a newer
    /// WhatsApp message arrives. Default: `false`.
    #[serde(default)]
    pub interrupt_on_new_message: bool,
    /// Usage mode for WhatsApp Web: "business" (default) or "personal".
    /// `dm_policy` and `group_policy` apply under BOTH modes. Personal mode
    /// additionally applies `self_chat_mode` and the fromMe handling; both are
    /// scoped to the personal branch by design, not by any protocol difference
    /// between the two modes.
    #[tab(Advanced)]
    #[serde(default)]
    pub mode: WhatsAppWebMode,
    /// Policy for direct messages, applied under both modes.
    /// "allowlist" (default) | "ignore" | "all".
    #[tab(Advanced)]
    #[serde(default)]
    pub dm_policy: WhatsAppChatPolicy,
    /// Policy for group chats, applied under both modes.
    /// "allowlist" (default) | "ignore" | "all".
    #[tab(Advanced)]
    #[serde(default)]
    pub group_policy: WhatsAppChatPolicy,
    /// When true and mode = "personal", always respond to messages in the
    /// user's own self-chat (Notes to Self). Defaults to false.
    #[tab(Advanced)]
    #[serde(default)]
    pub self_chat_mode: bool,
    /// Regex patterns for DM mention gating (case-insensitive).
    /// When non-empty, only direct messages matching at least one pattern are
    /// processed; matched fragments are stripped from the forwarded content.
    /// Example: `["@?OmnesAgent", "\\+?15555550123"]`
    #[tab(Advanced)]
    #[serde(default)]
    pub dm_mention_patterns: Vec<String>,
    /// Regex patterns for group-chat mention gating (case-insensitive).
    /// When non-empty, only group messages matching at least one pattern are
    /// processed; matched fragments are stripped from the forwarded content.
    /// Example: `["@?OmnesAgent", "\\+?15555550123"]`
    #[tab(Advanced)]
    #[serde(default)]
    pub group_mention_patterns: Vec<String>,
    /// Allowed group chats by JID (Web mode). An empty list (the default)
    /// admits NO group unless `group_policy = "all"`, which admits every
    /// group; a non-empty list drops every group message whose
    /// chat JID matches no entry. Each entry matches either the full group
    /// JID (`123456789012345@g.us`) or the JID user part - the segment before
    /// `@` (`123456789012345`) - compared exactly, not as a string prefix.
    /// Direct messages bypass this filter regardless of list contents.
    /// Modeled on the Matrix channel's `allowed_rooms`; it gates group
    /// identity, which `dm_policy`/`group_policy` (chat type) and the
    /// sender allowlist (sender) do not.
    #[tab(Advanced)]
    #[serde(default)]
    pub allowed_groups: Vec<String>,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Seconds to wait for operator approval on `always_ask` tools before auto-denying.
    #[tab(Behavior)]
    #[serde(default = "default_channel_approval_timeout_secs")]
    pub approval_timeout_secs: u64,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Per-(channel, recipient) outbound pacing floor in seconds.
    /// Range: `0..=REPLY_MIN_INTERVAL_MAX_SECS` (0 disables).
    #[serde(default)]
    pub reply_min_interval_secs: u64,
    /// Per-(channel, recipient) outbound pacing queue depth.
    /// Range: `0..=REPLY_QUEUE_DEPTH_CEILING`. When `reply_min_interval_secs > 0`
    /// and this value is `0`, the pacing wrapper substitutes
    /// `DEFAULT_REPLY_QUEUE_DEPTH` (16). When the queue is full, the
    /// newest send is dropped and a `WARN` is logged.
    #[serde(default)]
    pub reply_queue_depth_max: u16,
}

impl Default for WhatsAppConfig {
    /// Built by deserializing an empty object, so the serde defaults are the
    /// only source of truth and the two cannot disagree.
    ///
    /// A hand-written field list was the other option and was rejected: this
    /// struct carries two dozen fields, so such a list silently goes stale the
    /// first time one is added, which is the same class of defect this exists
    /// to remove. Every field here either declares a serde default or is an
    /// `Option`, which is what makes the empty object total. The unwrap is
    /// covered by `whatsapp_rust_default_matches_serde_default` below, so a
    /// field added without a default fails that test rather than reaching a
    /// caller.
    fn default() -> Self {
        serde_json::from_str("{}")
            .expect("every WhatsAppConfig field declares a serde default or is Option")
    }
}

impl ChannelConfig for WhatsAppConfig {
    fn name() -> &'static str {
        "WhatsApp"
    }
    fn desc() -> &'static str {
        "Business Cloud API"
    }
}

impl_reply_pacing!(
    TelegramConfig,
    DiscordConfig,
    SlackConfig,
    MattermostConfig,
    WebhookConfig,
    IMessageConfig,
    MatrixConfig,
    SignalConfig,
    WhatsAppConfig,
);

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.linq"]
pub struct LinqConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Linq Partner API token (Bearer auth)
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub api_token: String,
    /// Phone number to send from (E.164 format)
    #[tab(Advanced)]
    pub from_phone: String,
    /// Webhook signing secret for signature verification.
    ///
    /// Required to receive webhooks. Inbound requests are signature-verified,
    /// and with no secret configured the gateway cannot verify them, so it
    /// refuses them with `401` rather than accepting them unverified.
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub signing_secret: Option<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for LinqConfig {
    fn name() -> &'static str {
        "Linq"
    }
    fn desc() -> &'static str {
        "iMessage/RCS/SMS via Linq API"
    }
}

/// Nextcloud Talk bot configuration (webhook receive + signed bot send API).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.nextcloud_talk"]
pub struct NextcloudTalkConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Nextcloud base URL (e.g. `"https://cloud.example.com"`).
    #[tab(Connection)]
    pub base_url: String,
    /// Deprecated, unused. Nextcloud Talk sends do not authenticate via
    /// OCS bearer auth (see `webhook_secret`); this field is only accepted so
    /// existing configs that set it don't fail to parse. Safe to remove
    /// from your config.
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub app_token: Option<String>,
    /// DEPRECATED alias for `webhook_secret`, kept for migration.
    ///
    /// Nextcloud issues ONE secret per installed bot and uses it for both
    /// directions, so this cannot hold a different outbound secret. When both
    /// are set to different non-empty values the channel logs the conflict and
    /// fails closed as unconfigured: inbound `401` and no outbound send.
    /// Prefer `webhook_secret`; this alias will be removed.
    ///
    /// Upgrade from a `bot_token`-only configuration: copy the installed bot
    /// secret to `webhook_secret`, verify replies still send, then delete
    /// `bot_token`.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub bot_token: Option<String>,
    /// The bot secret Nextcloud installed for this bot. Canonical field.
    ///
    /// Used for BOTH directions: verifying inbound webhook signatures and
    /// signing outbound bot-API requests. When it is unset, inbound webhooks
    /// are rejected and no outbound request is sent (fail closed).
    ///
    /// Can also be set via `OMNESAGENT_NEXTCLOUD_TALK_WEBHOOK_SECRET`.
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub webhook_secret: Option<String>,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,
    /// Display name of the bot in Nextcloud Talk (e.g. "omnesagent").
    /// Used to filter out the bot's own messages and prevent feedback loops.
    /// If not set, defaults to an empty string (no self-message filtering by name).
    #[tab(Advanced)]
    #[serde(default)]
    pub bot_name: Option<String>,
    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
    /// Retained for configuration compatibility. Nextcloud Talk's bot API does
    /// not provide message IDs or edit/delete operations, so draft updates are
    /// disabled and responses are currently sent as one final message for every
    /// value.
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_mode: StreamMode,
    /// Retained for configuration compatibility. Currently inert while draft
    /// updates are disabled for this channel. Default: 1000 ms.
    #[tab(Behavior)]
    #[serde(default = "default_draft_update_interval_ms")]
    pub draft_update_interval_ms: u64,
}
impl NextcloudTalkConfig {
    /// Resolve the single bot secret Nextcloud installs for this bot.
    ///
    /// Nextcloud issues ONE secret per installed bot and uses it for both directions:
    /// verifying inbound webhook signatures and signing outbound bot-API requests. So
    /// both directions must resolve it identically, or one of them cannot match the
    /// installed bot.
    ///
    /// `webhook_secret` is canonical; `bot_token` is a deprecated alias kept for
    /// migration. Both are trimmed, and a blank value is treated as absent so an empty
    /// or whitespace alias cannot mask a real secret. Two conflicting non-empty values
    /// are rejected rather than silently split into two authorities.
    ///
    /// Returns `Ok(None)` when no secret is configured. Callers must fail closed on
    /// `Ok(None)` and on `Err`: send no outbound request, and reject inbound webhooks
    /// rather than skipping verification.
    pub fn resolve_bot_secret(&self) -> Result<Option<String>, NextcloudTalkSecretConflict> {
        let norm = |v: &Option<String>| -> Option<String> {
            v.as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(ToOwned::to_owned)
        };
        let canonical = norm(&self.webhook_secret);
        let alias = norm(&self.bot_token);
        match (canonical, alias) {
            (Some(a), Some(b)) if a != b => Err(NextcloudTalkSecretConflict),
            (Some(a), _) => Ok(Some(a)),
            (None, Some(b)) => Ok(Some(b)),
            (None, None) => Ok(None),
        }
    }
}

/// `webhook_secret` and the deprecated `bot_token` alias hold different non-empty
/// values. Nextcloud installs one secret per bot, so there is no correct choice
/// between them and guessing would leave one direction unable to match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NextcloudTalkSecretConflict;

impl std::fmt::Display for NextcloudTalkSecretConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "nextcloud_talk: `webhook_secret` and the deprecated `bot_token` alias are set \
to different values; Nextcloud installs one secret per bot. Set `webhook_secret` only.",
        )
    }
}

impl std::error::Error for NextcloudTalkSecretConflict {}

impl ChannelConfig for NextcloudTalkConfig {
    fn name() -> &'static str {
        "NextCloud Talk"
    }
    fn desc() -> &'static str {
        "NextCloud Talk platform"
    }
}

impl WhatsAppConfig {
    /// Detect which backend to use based on config fields.
    /// Cloud API when `phone_number_id` is set; otherwise Web when any
    /// Web-only selector (`session_path`, `pair_phone`, `pair_code`,
    /// `ws_url`, or `mode = personal`) is present. Falls back to Cloud
    /// for an otherwise-empty config (env-injected Cloud credentials).
    pub fn backend_type(&self) -> &'static str {
        if self.phone_number_id.is_some() {
            "cloud"
        } else if self.has_web_selector() {
            "web"
        } else {
            "cloud"
        }
    }

    /// Check if this is a valid Cloud API config
    pub fn is_cloud_config(&self) -> bool {
        self.phone_number_id.is_some() && self.access_token.is_some() && self.verify_token.is_some()
    }

    /// Any Web-only selector that signals WhatsApp Web intent. `mode`
    /// defaults to `Business` on every config, so only a non-default
    /// `Personal` mode counts; the rest are `Option` fields absent by
    /// default.
    pub fn has_web_selector(&self) -> bool {
        self.session_path.is_some()
            || self.pair_phone.is_some()
            || self.pair_code.is_some()
            || self.ws_url.is_some()
            || self.mode == WhatsAppWebMode::Personal
    }

    /// Check if this is a valid Web config. The Web client defaults a
    /// missing `session_path`, so any Web selector is sufficient.
    pub fn is_web_config(&self) -> bool {
        self.has_web_selector()
    }

    /// Returns true when both Cloud and Web selectors are present.
    ///
    /// Runtime currently prefers Cloud mode in this case for backward compatibility.
    pub fn is_ambiguous_config(&self) -> bool {
        self.phone_number_id.is_some() && self.has_web_selector()
    }
}

/// MQTT channel configuration (SOP listener).
///
/// Subscribes to MQTT topics and dispatches incoming messages
/// to the SOP engine for processing.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.mqtt"]
pub struct MqttConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// MQTT broker URL (e.g., `mqtt://localhost:1883` or `mqtts://broker.example.com:8883`).
    /// Use `mqtt://` for plain connections or `mqtts://` for TLS.
    #[tab(Connection)]
    pub broker_url: String,
    /// MQTT client ID (must be unique per broker).
    #[tab(Advanced)]
    pub client_id: String,
    /// Topics to subscribe to (e.g., `sensors/#`, `alerts/+/critical`).
    /// At least one topic is required.
    #[tab(Advanced)]
    #[serde(default)]
    pub topics: Vec<String>,
    /// MQTT QoS level (0 = at-most-once, 1 = at-least-once, 2 = exactly-once). Default: 1.
    #[tab(Advanced)]
    #[serde(default = "default_mqtt_qos")]
    pub qos: u8,
    /// Username for authentication (optional).
    #[tab(Connection)]
    pub username: Option<String>,
    /// Password for authentication (optional).
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub password: Option<String>,
    /// Enable TLS encryption. Must match the broker_url scheme:
    /// - `mqtt://` → `use_tls: false`
    /// - `mqtts://` → `use_tls: true`
    #[tab(Advanced)]
    #[serde(default)]
    pub use_tls: bool,
    /// Keep-alive interval in seconds (default: 30). Prevents broker disconnect on idle.
    #[tab(Advanced)]
    #[serde(default = "default_mqtt_keep_alive_secs")]
    pub keep_alive_secs: u64,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl MqttConfig {
    /// Validate the MQTT configuration.
    ///
    /// Checks:
    /// - QoS is 0, 1, or 2
    /// - broker_url uses valid scheme (`mqtt://` or `mqtts://`)
    /// - `use_tls` flag matches broker_url scheme
    /// - At least one topic is configured
    /// - client_id is non-empty
    pub fn validate(&self) -> anyhow::Result<()> {
        // QoS validation
        if self.qos > 2 {
            anyhow::bail!("qos must be 0, 1, or 2, got {}", self.qos);
        }

        // Broker URL validation
        let is_tls_scheme = self.broker_url.starts_with("mqtts://");
        let is_mqtt_scheme = self.broker_url.starts_with("mqtt://");

        if !is_tls_scheme && !is_mqtt_scheme {
            anyhow::bail!(
                "broker_url must start with 'mqtt://' or 'mqtts://', got: {}",
                self.broker_url
            );
        }

        // TLS flag validation
        if is_mqtt_scheme && self.use_tls {
            anyhow::bail!("use_tls is true but broker_url uses 'mqtt://' (not 'mqtts://')");
        }

        if is_tls_scheme && !self.use_tls {
            anyhow::bail!(
                "use_tls is false but broker_url uses 'mqtts://' (requires use_tls: true)"
            );
        }

        // Topics validation
        if self.topics.is_empty() {
            anyhow::bail!("at least one topic must be configured");
        }

        // Client ID validation
        if self.client_id.is_empty() {
            validation_bail!(
                RequiredFieldEmpty,
                "client_id",
                "client_id must not be empty"
            );
        }

        Ok(())
    }
}

impl ChannelConfig for MqttConfig {
    fn name() -> &'static str {
        "MQTT"
    }
    fn desc() -> &'static str {
        "MQTT SOP Listener"
    }
}

fn default_mqtt_qos() -> u8 {
    1
}

fn default_mqtt_keep_alive_secs() -> u64 {
    30
}

/// Filesystem SOP listener configuration.
///
/// Watches configured paths and dispatches file create/modify/delete/rename
/// events to the SOP engine. This is a fan-in listener, not a chat channel:
/// its `Channel::send` has no outbound surface.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.filesystem"]
pub struct FilesystemConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Root paths to watch. At least one is required.
    #[tab(Connection)]
    #[serde(default)]
    pub paths: Vec<String>,
    /// Watch nested paths beneath each root.
    #[tab(Behavior)]
    #[serde(default = "default_true")]
    pub recursive: bool,
    /// Glob patterns to include. Empty matches all paths under the roots.
    #[tab(Behavior)]
    #[serde(default)]
    pub include: Vec<String>,
    /// Glob patterns to exclude. Applied after `include`.
    #[tab(Behavior)]
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Event kinds to emit (`created`, `modified`, `deleted`, `renamed`).
    #[tab(Behavior)]
    #[serde(default = "default_filesystem_events")]
    pub events: Vec<String>,
    /// Collapse rapid repeated events per path/kind within this window.
    #[tab(Advanced)]
    #[serde(default = "default_filesystem_debounce_ms")]
    pub debounce_ms: u64,
    /// Wait this long after the last event before reading metadata or content,
    /// giving atomic writes and slow copies time to finish.
    #[tab(Advanced)]
    #[serde(default = "default_filesystem_settle_ms")]
    pub settle_ms: u64,
    /// Read file content into the event payload. Opt-in.
    #[tab(Behavior)]
    #[serde(default)]
    pub read_content: bool,
    /// Follow symlinks when reading event-path metadata, hash, and content.
    /// Off by default: a symlink under a watched root is rejected before any
    /// metadata/hash/content handling so its target cannot escape the root. When
    /// on, the canonical target must still resolve inside a configured root.
    #[tab(Advanced)]
    #[serde(default)]
    pub follow_symlinks: bool,
    /// Maximum file content bytes admitted into a payload when `read_content`,
    /// and the size ceiling for hashing. `Some(n)` caps reads and hashing at
    /// `n` bytes; oversize files are skipped rather than truncated. `None`
    /// removes the ceiling entirely (unbounded reads). Defaults to
    /// `Some(65536)` (64 KiB), so the cap applies by default whenever
    /// `read_content` or hashing runs.
    #[tab(Advanced)]
    #[serde(default = "default_filesystem_max_content_bytes")]
    pub max_content_bytes: Option<usize>,
    /// Watch broad system roots (`/`, `/home`, `/etc`, `/var`, `/proc`,
    /// `/sys`, `/dev`, `/tmp`) despite the deny-broad-roots default. The
    /// pseudo-filesystems `/proc` and `/sys` would surface kernel object
    /// paths in SOP payloads and flood the watcher with events. Off unless
    /// explicitly enabled.
    #[tab(Advanced)]
    #[serde(default)]
    pub allow_broad_roots: bool,
    /// Tools excluded from this channel's tool spec.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl Default for FilesystemConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            paths: Vec::new(),
            recursive: true,
            include: Vec::new(),
            exclude: Vec::new(),
            events: default_filesystem_events(),
            debounce_ms: default_filesystem_debounce_ms(),
            settle_ms: default_filesystem_settle_ms(),
            read_content: false,
            follow_symlinks: false,
            max_content_bytes: default_filesystem_max_content_bytes(),
            allow_broad_roots: false,
            excluded_tools: Vec::new(),
        }
    }
}

const FILESYSTEM_BROAD_ROOTS: [&str; 8] = [
    "/", "/home", "/etc", "/var", "/proc", "/sys", "/dev", "/tmp",
];

impl FilesystemConfig {
    /// Validate the filesystem listener configuration.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.paths.is_empty() {
            anyhow::bail!("at least one path must be configured");
        }
        for path in &self.paths {
            let trimmed = path.trim_end_matches('/');
            let normalized = if trimmed.is_empty() { "/" } else { trimmed };
            if !self.allow_broad_roots && FILESYSTEM_BROAD_ROOTS.contains(&normalized) {
                anyhow::bail!(
                    "path '{path}' is a broad system root; set allow_broad_roots = true to watch it"
                );
            }
        }
        for kind in &self.events {
            if !matches!(
                kind.as_str(),
                "created" | "modified" | "deleted" | "renamed"
            ) {
                anyhow::bail!(
                    "event '{kind}' is invalid; expected created, modified, deleted, or renamed"
                );
            }
        }
        Ok(())
    }
}

impl ChannelConfig for FilesystemConfig {
    fn name() -> &'static str {
        "Filesystem"
    }
    fn desc() -> &'static str {
        "Filesystem SOP Listener"
    }
}

fn default_filesystem_events() -> Vec<String> {
    vec![
        "created".to_string(),
        "modified".to_string(),
        "deleted".to_string(),
        "renamed".to_string(),
    ]
}

fn default_filesystem_debounce_ms() -> u64 {
    500
}

fn default_filesystem_settle_ms() -> u64 {
    250
}

fn default_filesystem_max_content_bytes() -> Option<usize> {
    Some(65536)
}

/// Generic AMQP 0-9-1 channel configuration (RabbitMQ, Fedora Messaging, etc.).
///
/// Subscribes to an exchange via routing keys and lifts each delivery into an
/// inbound `ChannelMessage`. The mapping from a JSON delivery body to message
/// fields is config-driven (`content_template`, `thread_id_field`) so a new
/// source — Anitya, an internal bus, anything publishing JSON — is onboarded by
/// configuration rather than code.
/// Where a fan-in delivery is routed once consumed. Used by AMQP, which carries
/// this field to choose between the agent loop, the SOP engine, or both.
/// Agent-loop channels (Telegram, Discord, Slack, ...) do not carry a `dispatch`
/// field: their fan-in is trigger-driven, opting in via a SOP `channel` trigger
/// while the normal agent turn always runs. The mode is source-agnostic.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SopDispatch {
    /// Drive a normal agent turn: the delivery becomes a `ChannelMessage`.
    /// This is the default and preserves each channel's original behavior.
    #[default]
    AgentLoop,
    /// Dispatch the delivery to the SOP engine as a `SopEvent` (`topic` =
    /// source identifier, `payload` = body), matching SOPs whose trigger for
    /// this source matches. No agent turn is started.
    Sop,
    /// Do both: dispatch to the SOP engine and drive an agent turn from the
    /// same delivery.
    SopAndAgentLoop,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.amqp"]
pub struct AmqpConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// AMQP broker URL. Use `amqp://` for plain or `amqps://` for TLS
    /// (e.g. `amqps://fedora:@rabbitmq.fedoraproject.org/%2Fpublic_pubsub`).
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub amqp_url: String,
    /// Exchange to bind the consumer queue to (e.g. `amq.topic`).
    #[tab(Advanced)]
    pub exchange: String,
    /// Routing keys to bind. Scope these to the topics of interest; binding
    /// `#` consumes the entire exchange and is almost never what you want.
    #[tab(Advanced)]
    #[serde(default)]
    pub routing_keys: Vec<String>,
    /// Queue name. Leave unset for a server-generated, transient,
    /// auto-deleted, exclusive queue. Set a stable name (UUID recommended)
    /// only when durable delivery across reconnects is required.
    #[tab(Advanced)]
    pub queue: Option<String>,
    /// Path to the CA certificate bundle for `amqps://` connections.
    #[tab(Connection)]
    pub ca_cert: Option<PathBuf>,
    /// Path to the client certificate for broker mutual-TLS auth
    /// (Fedora Messaging requires a client cert).
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub client_cert: Option<PathBuf>,
    /// Path to the client private key matching `client_cert`.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub client_key: Option<PathBuf>,
    /// Value placed in `ChannelMessage.sender` for every delivery from this
    /// source (e.g. `anitya`). Lets the orchestrator's self-loop guard and
    /// per-channel routing identify the origin.
    #[tab(Behavior)]
    #[serde(default = "default_amqp_sender_label")]
    pub sender_label: String,
    /// Template for the inbound message content. `{field}` placeholders are
    /// interpolated from the JSON delivery body's top-level keys. When empty,
    /// the raw delivery body is used verbatim.
    #[tab(Behavior)]
    #[serde(default)]
    pub content_template: String,
    /// Dotted path into the JSON delivery body whose value becomes the
    /// message `thread_ts`, correlating replies to the originating event
    /// (e.g. `message.project.name`). Empty disables threading.
    #[tab(Behavior)]
    #[serde(default)]
    pub thread_id_field: String,
    /// Acknowledgement mode. When `true` (default), deliveries are acked only
    /// after the message is durably handed to the agent loop, giving
    /// at-least-once semantics: a crash before hand-off redelivers the event.
    /// Set `false` for at-most-once (broker acks on dispatch), which silently
    /// drops in-flight events on crash and is only appropriate for
    /// non-side-effecting, drop-on-overload consumers.
    #[tab(Behavior)]
    #[serde(default = "default_amqp_durable_ack")]
    pub durable_ack: bool,
    /// Where consumed deliveries are routed: drive an agent turn
    /// (`agent_loop`, default), dispatch to the SOP engine (`sop`), or both
    /// (`sop_and_agent_loop`). The `sop` and `sop_and_agent_loop` modes match
    /// the delivery against SOP `amqp` triggers by routing key.
    #[tab(Behavior)]
    #[serde(default)]
    pub dispatch: SopDispatch,
    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl AmqpConfig {
    /// Validate the AMQP configuration.
    ///
    /// Checks:
    /// - `amqp_url` uses a valid scheme (`amqp://` or `amqps://`)
    /// - `amqps://` connections carry a CA certificate
    /// - `client_cert` and `client_key` are supplied together (mutual TLS)
    /// - the exchange is non-empty
    /// - at least one routing key is bound
    pub fn validate(&self) -> anyhow::Result<()> {
        let is_tls = self.amqp_url.starts_with("amqps://");
        let is_plain = self.amqp_url.starts_with("amqp://");

        if !is_tls && !is_plain {
            anyhow::bail!(
                "amqp_url must start with 'amqp://' or 'amqps://', got: {}",
                self.amqp_url
            );
        }

        if is_tls && self.ca_cert.is_none() {
            anyhow::bail!("amqps:// requires ca_cert to verify the broker");
        }

        match (self.client_cert.is_some(), self.client_key.is_some()) {
            (true, false) => {
                anyhow::bail!(
                    "client_cert is set but client_key is missing (both are required for mutual TLS)"
                )
            }
            (false, true) => {
                anyhow::bail!(
                    "client_key is set but client_cert is missing (both are required for mutual TLS)"
                )
            }
            _ => {}
        }

        if self.exchange.is_empty() {
            validation_bail!(RequiredFieldEmpty, "exchange", "exchange must not be empty");
        }

        if self.routing_keys.is_empty() {
            anyhow::bail!("at least one routing key must be configured");
        }

        Ok(())
    }
}

impl ChannelConfig for AmqpConfig {
    fn name() -> &'static str {
        "AMQP"
    }
    fn desc() -> &'static str {
        "AMQP topic consumer"
    }
}

fn default_amqp_sender_label() -> String {
    "amqp".to_string()
}

fn default_amqp_durable_ack() -> bool {
    true
}

/// IRC channel configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.irc"]
pub struct IrcConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// IRC server hostname
    #[tab(Advanced)]
    pub server: String,
    /// IRC server port (default: 6697 for TLS)
    #[tab(Advanced)]
    #[serde(default = "default_irc_port")]
    pub port: u16,
    /// Bot nickname
    #[tab(Advanced)]
    pub nickname: String,
    /// Username (defaults to nickname if not set)
    #[tab(Connection)]
    pub username: Option<String>,
    /// Channels to join on connect
    #[tab(Advanced)]
    #[serde(default)]
    pub channels: Vec<String>,
    /// Server password (for bouncers like ZNC)
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub server_password: Option<String>,
    /// NickServ IDENTIFY password
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub nickserv_password: Option<String>,
    /// SASL PLAIN password (IRCv3)
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub sasl_password: Option<String>,
    /// Verify TLS certificate (default: true)
    #[tab(Advanced)]
    pub verify_tls: Option<bool>,
    /// When true, only respond to messages that mention the bot.
    /// Other messages in the channel are silently ignored.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

impl ChannelConfig for IrcConfig {
    fn name() -> &'static str {
        "IRC"
    }
    fn desc() -> &'static str {
        "IRC over TLS"
    }
}

/// Twitch chat channel configuration. A thin adapter over IRC
/// (`irc.chat.twitch.tv:6697` over TLS); see the `twitch` channel module.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.twitch"]
pub struct TwitchConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false`.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Twitch login name of the bot account (case-insensitive — lowercased
    /// before send).
    #[tab(Connection)]
    pub bot_username: String,
    /// Twitch OAuth user-access token. The `oauth:` prefix is added
    /// automatically if missing, so both `"oauth:abcdef"` and `"abcdef"`
    /// work. Mint via <https://twitchapps.com/tmi/> or the Twitch CLI.
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub oauth_token: String,
    /// Twitch channels to join. Each entry receives a `#` prefix if missing
    /// and is lowercased before send (Twitch channel names are
    /// case-insensitive). E.g. `["mychannel", "#anotherchannel"]`.
    #[tab(Advanced)]
    #[serde(default)]
    pub channels: Vec<String>,
    /// When true, only respond to messages that mention the bot's login
    /// name. Default: `false`.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,
}

impl ChannelConfig for TwitchConfig {
    fn name() -> &'static str {
        "Twitch"
    }
    fn desc() -> &'static str {
        "Twitch chat (IRC)"
    }
}

fn default_irc_port() -> u16 {
    6697
}

/// How OmnesAgent receives events from Feishu / Lark.
///
/// - `websocket` (default) — persistent WSS long-connection; no public URL required.
/// - `webhook` — HTTP callback server; requires a public HTTPS endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, omnesagent_macros::ConfigEnum)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LarkReceiveMode {
    #[default]
    Websocket,
    Webhook,
}

/// Lark/Feishu configuration for messaging integration.
/// Lark is the international version; Feishu is the Chinese version.
///
/// `Default` is implemented below rather than derived. A derived `Default`
/// zeroes every field, which disagrees with the serde defaults, and for
/// `approval_timeout_secs` that disagreement is load-bearing: `0` is an
/// already-elapsed deadline, so an alias built in Rust would deny every
/// approval while an alias parsed from a file waits the documented 300s.
#[derive(Debug, Clone, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.lark"]
pub struct LarkConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// App ID from Lark/Feishu developer console
    #[tab(Connection)]
    pub app_id: String,
    /// App Secret from Lark/Feishu developer console
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub app_secret: String,
    /// Encrypt key for webhook message decryption and signed event-subscription
    /// validation (optional when verification_token is configured for plaintext
    /// callbacks).
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub encrypt_key: Option<String>,
    /// Verification token for plaintext webhook validation and URL verification.
    /// Required in webhook mode unless encrypt_key is configured for signed
    /// event-subscription callbacks; optional in websocket mode.
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub verification_token: Option<String>,
    /// When true, only respond to messages that @-mention the bot in groups.
    /// Direct messages are always processed.
    #[tab(Behavior)]
    #[serde(default)]
    pub mention_only: bool,
    /// Whether to use the Feishu (Chinese) endpoint instead of Lark (International)
    #[tab(Advanced)]
    #[serde(default)]
    pub use_feishu: bool,
    /// Event receive mode: "websocket" (default) or "webhook"
    #[tab(Advanced)]
    #[serde(default)]
    pub receive_mode: LarkReceiveMode,
    /// HTTP port for webhook mode only. Must be set when receive_mode = "webhook".
    /// Not required (and ignored) for websocket mode.
    #[tab(Advanced)]
    #[serde(default)]
    pub port: Option<u16>,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,

    /// Time in seconds an approval card waits for user response before
    /// the runtime auto-denies. Default: 300 (5 minutes).
    #[tab(Behavior)]
    #[serde(default = "default_channel_approval_timeout_secs")]
    pub approval_timeout_secs: u64,
    /// When `true`, group-chat sessions key on the sender's open_id, so
    /// distinct members of the same group chat don't share conversation
    /// context. When `false` (default), all members of a group share one
    /// session keyed on chat_id (matches the existing behavior). 1-on-1
    /// chats are unaffected (chat_id is already unique per user-bot pair).
    #[tab(Behavior)]
    #[serde(default)]
    pub per_user_session: bool,

    /// Override for the top-level `ack_reactions` setting. When `None`, the
    /// channel falls back to `[channels].ack_reactions`. When set
    /// explicitly, it takes precedence.
    #[tab(Behavior)]
    #[serde(default)]
    pub ack_reactions: Option<bool>,

    /// Streaming mode for the LLM response: `off` (default) routes every
    /// response through `send()`; `partial` opens a Feishu interactive
    /// card and edits it incrementally via `update_draft` /
    /// `finalize_draft`; `multi_message` is rejected for Lark (Feishu has
    /// no equivalent surface — falls back to `off` with a warning).
    #[tab(Behavior)]
    #[serde(default)]
    pub stream_mode: StreamMode,

    /// Minimum interval between consecutive `update_draft` PATCH calls in
    /// milliseconds. Default 1000 ms tunes to Feishu's 5 QPS-per-message
    /// edit cap; raise on enterprise plans with higher quotas.
    #[tab(Behavior)]
    #[serde(default = "default_draft_update_interval_ms")]
    pub draft_update_interval_ms: u64,
}

impl Default for LarkConfig {
    /// Built by deserializing an object supplying only `app_id` and
    /// `app_secret`, the two fields with no serde default - a Lark channel
    /// must always name its app credentials, so they stay required rather
    /// than gaining one. Every other field either declares a serde default
    /// or is an `Option`, so this is the minimal object that makes the
    /// deserialize total; those defaults stay the only source of truth for
    /// everything but `app_id` and `app_secret`.
    fn default() -> Self {
        serde_json::from_str(r#"{"app_id":"","app_secret":""}"#)
            .expect("every other LarkConfig field declares a serde default or is Option")
    }
}

impl ChannelConfig for LarkConfig {
    fn name() -> &'static str {
        "Lark"
    }
    fn desc() -> &'static str {
        "Lark Bot"
    }
}

/// DM (1:1 chat) access policy for the LINE channel.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LineDmPolicy {
    /// Respond to every DM regardless of who sent it.
    Open,
    /// Require a one-time `/bind <code>` handshake before responding (default).
    /// OmnesAgent prints the bind code on startup; send it once to unlock access.
    #[default]
    Pairing,
    /// Respond only to LINE user IDs listed in `allowed_users`.
    Allowlist,
}

/// Group / multi-person chat policy for the LINE channel.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, omnesagent_macros::ConfigEnum,
)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum LineGroupPolicy {
    /// Respond to every message in group/room chats.
    Open,
    /// Respond only when the bot is @mentioned (default).
    #[default]
    Mention,
    /// Ignore all messages in group/room chats.
    Disabled,
}

/// LINE Messaging API channel configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Configurable)]
#[cfg_attr(feature = "schema-export", derive(schemars::JsonSchema))]
#[prefix = "channels.line"]
pub struct LineConfig {
    /// Whether this channel is active. The runtime only loads channels whose
    /// `enabled = true`. Default: `false` so an operator who pastes a partial
    /// `[channels.<type>.<alias>]` block doesn't accidentally bring a channel
    /// live before the rest of its config is filled in.
    #[tab(Behavior)]
    #[serde(default)]
    pub enabled: bool,
    /// Long-lived channel access token (from LINE Developers Console).
    /// Used for both the Reply API and the Push API fallback.
    /// Falls back to the `LINE_CHANNEL_ACCESS_TOKEN` environment variable if empty.
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub channel_access_token: String,
    /// Channel secret (from LINE Developers Console).
    /// Used to verify the `X-Line-Signature` header on incoming webhooks.
    /// Falls back to the `LINE_CHANNEL_SECRET` environment variable if empty.
    #[serde(default)]
    #[secret]
    #[tab(Connection)]
    #[cfg_attr(feature = "schema-export", schemars(extend("x-secret" = true)))]
    pub channel_secret: String,
    /// DM (1:1 chat) access policy. Default: `pairing`.
    ///
    /// - `open`: respond to everyone
    /// - `pairing`: require one-time `/bind <code>` handshake on first contact
    /// - `allowlist`: respond only to user IDs listed in `allowed_users`
    #[tab(Advanced)]
    #[serde(default)]
    pub dm_policy: LineDmPolicy,
    /// Group / multi-person chat policy. Default: `mention`.
    ///
    /// - `open`: respond to every message
    /// - `mention`: respond only when @mentioned
    /// - `disabled`: ignore all group messages
    #[tab(Advanced)]
    #[serde(default)]
    pub group_policy: LineGroupPolicy,
    /// TCP port the embedded webhook server listens on. Default: `8443`.
    #[tab(Advanced)]
    #[serde(default = "default_line_webhook_port")]
    pub webhook_port: u16,
    /// Per-channel proxy URL (http, https, socks5, socks5h).
    /// Overrides the global `[proxy]` setting for this channel only.
    #[tab(Advanced)]
    #[serde(default)]
    pub proxy_url: Option<String>,

    /// Display name shown as the sender in LINE chat bubbles.
    /// Defaults to `"AI"` when unset or empty.
    #[tab(Behavior)]
    #[serde(default)]
    pub sender_name: Option<String>,

    /// Tools excluded from this channel's tool spec. When set, these tools
    /// are not exposed to the model when responding via this channel.
    #[tab(Behavior)]
    #[serde(default)]
    pub excluded_tools: Vec<String>,
}

fn default_line_webhook_port() -> u16 {
    8443
}

impl ChannelConfig for LineConfig {
    fn name() -> &'static str {
        "LINE"
    }
    fn desc() -> &'static str {
        "connect your LINE bot"
    }
}
