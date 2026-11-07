use crate::core::model::{CropMode, PrintSize, WatermarkConfig};
use crate::core::{geometry, watermark};
use image::{imageops, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSettings {
    pub size: PrintSize,
    pub crop_mode: CropMode,
    pub border_enabled: bool,
    pub border_color: [u8; 3],
    pub border_width_mm: f32,
    pub watermark: WatermarkConfig,
    pub quality: u8,
    pub format: String,
}

pub fn decode_image(input: &Path) -> Result<RgbaImage, String> {
    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());
    match ext.as_deref() {
        Some("heic") | Some("heif") => crate::core::heic::decode(input),
        _ => image::open(input)
            .map_err(|e| format!("解码失败: {e}"))
            .map(|d| d.to_rgba8()),
    }
}

fn apply_orientation(img: RgbaImage, orientation: u16) -> RgbaImage {
    match orientation {
        3 => imageops::rotate180(&img),
        6 => imageops::rotate90(&img),
        8 => imageops::rotate270(&img),
        2 => imageops::flip_horizontal(&img),
        4 => imageops::flip_vertical(&img),
        5 => imageops::rotate90(&imageops::flip_horizontal(&img)),
        7 => imageops::rotate270(&imageops::flip_horizontal(&img)),
        _ => img,
    }
}

/// 解码并按 EXIF 朝向旋正，供预览/裁切代理图与导出共用。
pub fn load_oriented(input: &Path, orientation: u16) -> Result<RgbaImage, String> {
    Ok(apply_orientation(decode_image(input)?, orientation))
}

fn render_overlay(
    w: u32,
    h: u32,
    cfg: &WatermarkConfig,
    date: &str,
    place: &str,
) -> Option<RgbaImage> {
    const FONT: &[u8] = include_bytes!("../../assets/fonts/Songti.ttf");
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_font_data(FONT.to_vec());

    let family = opt
        .fontdb
        .faces()
        .next()
        .and_then(|f| f.families.first().map(|(n, _)| n.clone()))
        .unwrap_or_else(|| cfg.font.clone());

    let mut cfg2 = cfg.clone();
    cfg2.font = family;
    let svg = watermark::build_svg(w, h, &cfg2, date, place);

    let tree = usvg::Tree::from_str(&svg, &opt).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let mut data = pixmap.data().to_vec();
    for px in data.chunks_mut(4) {
        let a = px[3] as u32;
        if a > 0 && a < 255 {
            px[0] = ((px[0] as u32 * 255) / a).min(255) as u8;
            px[1] = ((px[1] as u32 * 255) / a).min(255) as u8;
            px[2] = ((px[2] as u32 * 255) / a).min(255) as u8;
        }
    }
    RgbaImage::from_raw(w, h, data)
}

pub fn compose(
    input: &Path,
    s: &ResolvedSettings,
    date: &str,
    place: &str,
    orientation: u16,
    focus: (f32, f32),
) -> Result<RgbaImage, String> {
    let source = load_oriented(input, orientation)?;
    let (tw, th) = s.size.pixels();

    let mut canvas: RgbaImage = match s.crop_mode {
        CropMode::Fill => {
            let (x, y, cw, ch) =
                geometry::fill_rect(source.width(), source.height(), tw, th, focus);
            let cropped = imageops::crop_imm(&source, x, y, cw, ch).to_image();
            imageops::resize(&cropped, tw, th, imageops::FilterType::Lanczos3)
        }
        CropMode::Fit => {
            let mut canvas = RgbaImage::from_pixel(
                tw,
                th,
                Rgba([
                    s.border_color[0],
                    s.border_color[1],
                    s.border_color[2],
                    255,
                ]),
            );
            let (fx, fy, fw, fh) = geometry::fit_rect(source.width(), source.height(), tw, th);
            let resized = imageops::resize(&source, fw, fh, imageops::FilterType::Lanczos3);
            imageops::overlay(&mut canvas, &resized, fx as i64, fy as i64);
            canvas
        }
    };

    if s.watermark.show_date || s.watermark.show_location {
        if let Some(overlay) = render_overlay(tw, th, &s.watermark, date, place) {
            imageops::overlay(&mut canvas, &overlay, 0, 0);
        }
    }

    Ok(canvas)
}

pub fn process(
    input: &Path,
    output: &Path,
    s: &ResolvedSettings,
    date: &str,
    place: &str,
    orientation: u16,
    focus: (f32, f32),
) -> Result<(), String> {
    let final_img = compose(input, s, date, place, orientation, focus)?;

    if s.format == "png" {
        final_img.save(output).map_err(|e| format!("编码失败: {e}"))?;
    } else {
        let file = std::fs::File::create(output).map_err(|e| format!("创建文件失败: {e}"))?;
        let mut writer = std::io::BufWriter::new(file);
        let rgb = image::DynamicImage::ImageRgba8(final_img).to_rgb8();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, s.quality);
        enc.encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("JPEG 编码失败: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{CropMode, PrintSize, WatermarkConfig};
    use image::{Rgba, RgbaImage};

    fn settings() -> ResolvedSettings {
        ResolvedSettings {
            size: PrintSize {
                width_mm: 101.6,
                height_mm: 152.4,
                dpi: 300,
            },
            crop_mode: CropMode::Fill,
            border_enabled: false,
            border_color: [255, 255, 255],
            border_width_mm: 0.0,
            watermark: WatermarkConfig::default(),
            quality: 92,
            format: "png".into(),
        }
    }

    #[test]
    fn watermark_overlay_draws_text_pixels() {
        let cfg = WatermarkConfig::default();
        let overlay = render_overlay(1200, 1800, &cfg, "2024 · 六月", "杭州市").unwrap();
        let count = overlay.pixels().filter(|p| p.0[3] > 16).count();
        assert!(count > 500, "overlay too sparse, text likely missing: {count}");
    }

    #[test]
    fn output_has_target_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.png");
        let mut img = RgbaImage::new(3000, 2000);
        for p in img.pixels_mut() {
            *p = Rgba([120, 150, 100, 255]);
        }
        img.save(&input).unwrap();

        let output = dir.path().join("out.png");
        process(
            &input,
            &output,
            &settings(),
            "2024 · 六月",
            "杭州市",
            1,
            (0.5, 0.5),
        )
        .unwrap();

        let out = image::open(&output).unwrap().to_rgba8();
        assert_eq!(out.dimensions(), (1200, 1800));
    }

    #[test]
    fn focus_changes_crop_region() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.png");
        let mut img = RgbaImage::new(3000, 2000);
        for (x, _y, p) in img.enumerate_pixels_mut() {
            *p = if x < 1500 {
                Rgba([230, 20, 20, 255])
            } else {
                Rgba([20, 20, 230, 255])
            };
        }
        img.save(&input).unwrap();

        let left_out = dir.path().join("left.png");
        process(&input, &left_out, &settings(), "", "", 1, (0.0, 0.5)).unwrap();
        let right_out = dir.path().join("right.png");
        process(&input, &right_out, &settings(), "", "", 1, (1.0, 0.5)).unwrap();

        let left = image::open(&left_out).unwrap().to_rgba8();
        let right = image::open(&right_out).unwrap().to_rgba8();
        let lp = left.get_pixel(10, 900);
        let rp = right.get_pixel(10, 900);
        assert!(lp.0[0] > 180 && lp.0[2] < 80, "left crop should be red: {lp:?}");
        assert!(rp.0[2] > 180 && rp.0[0] < 80, "right crop should be blue: {rp:?}");
    }

    #[test]
    fn jpeg_output_is_written_with_target_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.png");
        let mut img = RgbaImage::new(800, 600);
        for p in img.pixels_mut() {
            *p = Rgba([90, 120, 160, 255]);
        }
        img.save(&input).unwrap();

        let mut s = settings();
        s.format = "jpeg".into();
        s.quality = 90;
        let output = dir.path().join("out.jpg");
        process(&input, &output, &s, "2024 · 六月", "杭州市", 1, (0.5, 0.5)).unwrap();

        assert!(output.exists());
        assert!(std::fs::metadata(&output).unwrap().len() > 1000);
        let out = image::open(&output).unwrap();
        assert_eq!(out.width(), 1200);
        assert_eq!(out.height(), 1800);
    }

    #[test]
    fn fit_mode_paints_border_color() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.png");
        let mut img = RgbaImage::new(3000, 2000);
        for p in img.pixels_mut() {
            *p = Rgba([0, 0, 0, 255]);
        }
        img.save(&input).unwrap();

        let mut s = settings();
        s.crop_mode = CropMode::Fit;
        s.border_enabled = true;
        s.border_color = [255, 0, 0];
        let output = dir.path().join("out.png");
        process(&input, &output, &s, "", "", 1, (0.5, 0.5)).unwrap();

        let out = image::open(&output).unwrap().to_rgba8();
        let top = out.get_pixel(600, 300);
        assert_eq!(top.0[0], 255);
        assert_eq!(top.0[1], 0);
    }
}
