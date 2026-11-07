use exif::{In, Reader, Tag, Value};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub struct ExifInfo {
    pub datetime: Option<String>,
    pub gps: Option<(f64, f64)>,
    pub orientation: u16,
}

impl Default for ExifInfo {
    fn default() -> Self {
        Self {
            datetime: None,
            gps: None,
            orientation: 1,
        }
    }
}

pub fn gps_to_decimal(
    lat: (f64, f64, f64),
    lng: (f64, f64, f64),
    lat_ref: char,
    lng_ref: char,
) -> (f64, f64) {
    let to_deg = |d: (f64, f64, f64)| d.0 + d.1 / 60.0 + d.2 / 3600.0;
    let mut la = to_deg(lat);
    let mut ln = to_deg(lng);
    if lat_ref == 'S' {
        la = -la;
    }
    if lng_ref == 'W' {
        ln = -ln;
    }
    (la, ln)
}

fn rational_triplet(exif: &exif::Exif, tag: Tag) -> Option<(f64, f64, f64)> {
    match exif.get_field(tag, In::PRIMARY)?.value {
        Value::Rational(ref v) if v.len() >= 3 => {
            Some((v[0].to_f64(), v[1].to_f64(), v[2].to_f64()))
        }
        _ => None,
    }
}

fn ascii_char(field: &exif::Field) -> Option<char> {
    match &field.value {
        Value::Ascii(v) => v.first().and_then(|b| b.first()).map(|b| *b as char),
        _ => None,
    }
}

pub fn parse(bytes: &[u8]) -> ExifInfo {
    let mut info = ExifInfo::default();
    let exif = match Reader::new().read_from_container(&mut Cursor::new(bytes)) {
        Ok(e) => e,
        Err(_) => return info,
    };

    if let Some(f) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
        info.datetime = Some(
            f.display_value()
                .to_string()
                .trim_matches('"')
                .to_string(),
        );
    }

    if let (Some(lat), Some(lng)) = (
        rational_triplet(&exif, Tag::GPSLatitude),
        rational_triplet(&exif, Tag::GPSLongitude),
    ) {
        let lat_ref = exif
            .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
            .and_then(ascii_char)
            .unwrap_or('N');
        let lng_ref = exif
            .get_field(Tag::GPSLongitudeRef, In::PRIMARY)
            .and_then(ascii_char)
            .unwrap_or('E');
        info.gps = Some(gps_to_decimal(lat, lng, lat_ref, lng_ref));
    }

    if let Some(f) = exif.get_field(Tag::Orientation, In::PRIMARY) {
        if let Value::Short(ref v) = f.value {
            if let Some(o) = v.first() {
                info.orientation = *o;
            }
        }
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gps_north_east_positive() {
        let v = gps_to_decimal((30.0, 14.0, 24.0), (120.0, 9.0, 0.0), 'N', 'E');
        assert!((v.0 - 30.24).abs() < 1e-6);
        assert!((v.1 - 120.15).abs() < 1e-6);
    }

    #[test]
    fn gps_south_west_negative() {
        let v = gps_to_decimal((33.0, 51.0, 0.0), (151.0, 12.0, 0.0), 'S', 'W');
        assert!((v.0 + 33.85).abs() < 1e-6);
        assert!((v.1 + 151.2).abs() < 1e-6);
    }

    #[test]
    fn garbage_returns_defaults() {
        let info = parse(&[0u8, 1, 2, 3, 4]);
        assert!(info.datetime.is_none());
        assert!(info.gps.is_none());
        assert_eq!(info.orientation, 1);
    }
}
