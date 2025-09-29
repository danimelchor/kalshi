use bincode::{self, Decode, Encode};

#[derive(Encode, Decode, Debug, Clone)]
pub enum Temperature {
    Celsius(f32),
    Fahrenheit(f32),
    Kelvin(f32),
}

impl Temperature {
    pub fn as_celsius(&self) -> f32 {
        match *self {
            Temperature::Celsius(c) => c,
            Temperature::Fahrenheit(f) => (f - 32.0) * 5.0 / 9.0,
            Temperature::Kelvin(k) => k - 273.15,
        }
    }

    pub fn as_fahrenheit(&self) -> f32 {
        match *self {
            Temperature::Celsius(c) => (c * 9.0 / 5.0) + 32.0,
            Temperature::Fahrenheit(f) => f,
            Temperature::Kelvin(k) => (k - 273.15) * 9.0 / 5.0 + 32.0,
        }
    }

    pub fn as_kelvin(&self) -> f32 {
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
