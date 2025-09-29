#[derive(Debug)]
pub struct LatLon(f32, f32);

impl LatLon {
    pub fn new(lat: f32, lon: f32) -> LatLon {
        LatLon(lat, lon)
    }

    pub fn euclidean_sq<T: Into<LatLon>>(&self, other: T) -> f32 {
        let other: LatLon = other.into();
        let dlat = self.0 - other.0;
        let dlon = self.1 - other.1;
        dlat * dlat + dlon * dlon
    }
}

impl From<&(f32, f32)> for LatLon {
    fn from(t: &(f32, f32)) -> Self {
        LatLon(t.0, t.1)
    }
}
