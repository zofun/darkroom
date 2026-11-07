use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PlaceEntry {
    pub name: String,
    pub lat: f64,
    pub lng: f64,
}

pub struct PlaceDb {
    entries: Vec<PlaceEntry>,
}

fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng / 2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}

impl PlaceDb {
    pub fn from_entries(entries: Vec<PlaceEntry>) -> Self {
        Self { entries }
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        Ok(Self {
            entries: serde_json::from_str(json)?,
        })
    }

    pub fn embedded() -> Self {
        const JSON: &str = include_str!("../../assets/geo/places.json");
        Self::from_json(JSON).expect("embedded place db is valid json")
    }

    pub fn nearest(&self, lat: f64, lng: f64) -> Option<&PlaceEntry> {
        self.entries
            .iter()
            .map(|e| (e, haversine_km(lat, lng, e.lat, e.lng)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(e, _)| e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> PlaceDb {
        PlaceDb::from_entries(vec![
            PlaceEntry {
                name: "北京市".into(),
                lat: 39.90,
                lng: 116.40,
            },
            PlaceEntry {
                name: "杭州市".into(),
                lat: 30.27,
                lng: 120.15,
            },
            PlaceEntry {
                name: "上海市".into(),
                lat: 31.23,
                lng: 121.47,
            },
        ])
    }

    #[test]
    fn nearest_hangzhou() {
        assert_eq!(db().nearest(30.24, 120.15).unwrap().name, "杭州市");
    }

    #[test]
    fn nearest_beijing() {
        assert_eq!(db().nearest(39.92, 116.39).unwrap().name, "北京市");
    }

    #[test]
    fn empty_none() {
        assert!(PlaceDb::from_entries(vec![]).nearest(0.0, 0.0).is_none());
    }
}
