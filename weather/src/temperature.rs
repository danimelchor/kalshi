use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Temperature {
    Celsius(f64),
    Fahrenheit(f64),
    Kelvin(f64),
}

impl Temperature {
    pub fn as_celsius(&self) -> f64 {
        match *self {
            Temperature::Celsius(c) => c,
            Temperature::Fahrenheit(f) => (f - 32.0) * 5.0 / 9.0,
            Temperature::Kelvin(k) => k - 273.15,
        }
    }

    pub fn as_fahrenheit(&self) -> f64 {
        match *self {
            Temperature::Celsius(c) => (c * 9.0 / 5.0) + 32.0,
            Temperature::Fahrenheit(f) => f,
            Temperature::Kelvin(k) => (k - 273.15) * 9.0 / 5.0 + 32.0,
        }
    }

    pub fn as_kelvin(&self) -> f64 {
        match *self {
            Temperature::Celsius(c) => c + 273.15,
            Temperature::Fahrenheit(f) => (f - 32.0) * 5.0 / 9.0 + 273.15,
            Temperature::Kelvin(k) => k,
        }
    }

    /// Return a new `Temperature` in Celsius
    pub fn to_celsius(&self) -> Temperature {
        Temperature::Celsius(self.as_celsius())
    }

    /// Return a new `Temperature` in Fahrenheit
    pub fn to_fahrenheit(&self) -> Temperature {
        Temperature::Fahrenheit(self.as_fahrenheit())
    }

    /// Return a new `Temperature` in Kelvin
    pub fn to_kelvin(&self) -> Temperature {
        Temperature::Kelvin(self.as_kelvin())
    }
}

impl PartialEq for Temperature {
    fn eq(&self, other: &Self) -> bool {
        self.as_kelvin() == other.as_kelvin()
    }
}
impl Eq for Temperature {}

impl Ord for Temperature {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_kelvin()
            .partial_cmp(&other.as_kelvin())
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Temperature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
