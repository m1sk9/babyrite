//! State shared by every event handler task.

/// Clients the handlers need, shared behind an `Arc` across event tasks.
pub struct BotContext {
    /// Discord REST API client.
    pub http: twilight_http::Client,
    /// Shared client for raw.githubusercontent.com fetches.
    pub github: reqwest::Client,
}
