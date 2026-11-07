use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CropMode {
    Fill,
    Fit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WatermarkStyle {
    Engraved,
    Siderail,
    Cornernote,
    Footerband,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WmPosition {
    Br,
    Bl,
    Tr,
    Tl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PrintSize {
    pub width_mm: f32,
    pub height_mm: f32,
    pub dpi: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkConfig {
    pub style: WatermarkStyle,
    pub show_date: bool,
    pub show_location: bool,
    pub show_time: bool,
    pub date_format: String,
    pub font: String,
    pub color_token: String,
    pub readability: String,
    pub scale: f32,
    pub position: WmPosition,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            style: WatermarkStyle::Cornernote,
            show_date: true,
            show_location: true,
            show_time: false,
            date_format: "YYYY · M月".into(),
            font: "Songti".into(),
            color_token: "warm".into(),
            readability: "none".into(),
            scale: 1.0,
            position: WmPosition::Bl,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportConfig {
    pub format: String,
    pub quality: u8,
    pub filename_pattern: String,
    pub keep_exif: bool,
    pub strip_gps: bool,
    pub output_dir: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: "jpeg".into(),
            quality: 92,
            filename_pattern: "{name}_{size}".into(),
            keep_exif: true,
            strip_gps: true,
            output_dir: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub size: PrintSize,
    pub crop_mode: CropMode,
    pub border_enabled: bool,
    pub border_color: String,
    pub border_width_mm: f32,
    pub watermark: WatermarkConfig,
    pub export: ExportConfig,
}
