use std::borrow::Cow;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::SyncStatus;

/// Server health status
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Health {
    /// Health status ("healthy")
    pub status: Cow<'static, str>,
    /// Service name
    pub service: Cow<'static, str>,
    /// Server version
    pub version: Cow<'static, str>,
    /// Active chain ("bitcoin" or "dogecoin")
    pub chain: Cow<'static, str>,
    /// Active chain ticker symbol ("BTC" or "DOGE")
    pub ticker: Cow<'static, str>,
    /// Active chain display name ("Bitcoin" or "Dogecoin")
    pub coin_name: Cow<'static, str>,
    /// Current server time (ISO 8601)
    pub timestamp: String,
    /// Server start time (ISO 8601)
    pub started_at: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Sync status
    #[serde(flatten)]
    pub sync: SyncStatus,
}
