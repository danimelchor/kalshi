use anyhow;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTicker(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTicker(String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesTicker(String);

macro_rules! impl_from_string {
    ($($t:ident),+ $(,)?) => {
        $(
            impl From<$t> for String {
                fn from(value: $t) -> Self {
                    value.0
                }
            }

            impl From<String> for $t {
                fn from(value: String) -> Self {
                    $t(value.to_uppercase())
                }
            }

            impl FromStr for $t {
                type Err = anyhow::Error;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let s: String = s.into();
                    Ok($t(s.to_uppercase()))
                }
            }


            impl Display for $t {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.0)
                }
            }
        )+
    };
}

impl_from_string!(MarketTicker, EventTicker, SeriesTicker);
