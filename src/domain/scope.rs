use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumString, Display)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum AudienceScope {
    Public,
    Restrained,
    Private,
}

impl AudienceScope {
    pub fn all() -> &'static [AudienceScope] {
        &[
            AudienceScope::Public,
            AudienceScope::Restrained,
            AudienceScope::Private,
        ]
    }
}

impl Default for AudienceScope {
    fn default() -> Self {
        AudienceScope::Public
    }
}
