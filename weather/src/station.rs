use crate::coords::LatLon;

pub enum Station {
    KNYC,
}

impl Station {
    pub fn latlon(&self) -> LatLon {
        match self {
            Station::KNYC => LatLon::new(40.78333, -73.96667),
        }
    }
}
