pub struct MarketTicker(String);

pub struct EventTicker(String);

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
                    $t(value)
                }
            }
        )+
    };
}

impl_from_string!(MarketTicker, EventTicker, SeriesTicker);
