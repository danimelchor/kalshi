use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub struct Money(Decimal);

#[derive(Debug, Clone)]
pub struct Price(Decimal);

macro_rules! impl_money {
    ($($t:ident),+ $(,)?) => {
        $(
            impl Serialize for $t {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serializer.serialize_str(&self.0.to_string())
                }
            }

            impl<'de> Deserialize<'de> for $t {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    let s = String::deserialize(deserializer)?;
                    let dec = s.parse::<Decimal>().map_err(serde::de::Error::custom)?;
                    Ok($t(dec))
                }
            }
        )+
    }
}

impl_money!(Money, Price);

#[macro_export]
macro_rules! usd {
    ($val:expr) => {
        Money(Decimal::from_str(stringify!($val)).unwrap())
    };
}
