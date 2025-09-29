use crate::station::Station;

pub enum Model {
    HRRR,
}

impl Model {
    pub fn computed_grid_location_and_info(
        &self,
        station: &Station,
    ) -> ((usize, usize), (usize, usize)) {
        match (self, station) {
            (Model::HRRR, Station::KNYC) => ((1553, 698), (1799, 1059)),
        }
    }
}
