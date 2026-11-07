use crate::core::model::PrintSize;

pub const MM_PER_INCH: f32 = 25.4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Preset {
    Five,
    Six,
    Seven,
}

pub fn preset(p: Preset) -> PrintSize {
    let (w, h) = match p {
        Preset::Five => (3.5, 5.0),
        Preset::Six => (4.0, 6.0),
        Preset::Seven => (5.0, 7.0),
    };
    PrintSize {
        width_mm: w * MM_PER_INCH,
        height_mm: h * MM_PER_INCH,
        dpi: 300,
    }
}

impl PrintSize {
    pub fn pixels(&self) -> (u32, u32) {
        let w = (self.width_mm / MM_PER_INCH * self.dpi as f32).round() as u32;
        let h = (self.height_mm / MM_PER_INCH * self.dpi as f32).round() as u32;
        (w.max(1), h.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_inch_at_300dpi_is_1200x1800() {
        assert_eq!(preset(Preset::Six).pixels(), (1200, 1800));
    }

    #[test]
    fn five_inch_at_300dpi_is_1050x1500() {
        assert_eq!(preset(Preset::Five).pixels(), (1050, 1500));
    }

    #[test]
    fn seven_inch_at_300dpi_is_1500x2100() {
        assert_eq!(preset(Preset::Seven).pixels(), (1500, 2100));
    }

    #[test]
    fn custom_mm_pixels() {
        let s = PrintSize {
            width_mm: 100.0,
            height_mm: 150.0,
            dpi: 300,
        };
        assert_eq!(s.pixels(), (1181, 1772));
    }
}
