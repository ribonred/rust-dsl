use serde::{Deserialize, Serialize};
use strum::VariantNames;

#[derive(Debug, Clone, Serialize, Deserialize, VariantNames)]
pub enum Import {
    #[serde(rename = "option")]
    Option(String),
    #[serde(rename = "multi_option")]
    MultiOption(String),
    #[serde(rename = "matching_pair")]
    MatchingPair(String),
}
