use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FeedFilter {
    Global,
    Following,
    Me,
}

impl Default for FeedFilter {
    fn default() -> Self {
        FeedFilter::Global
    }
}
