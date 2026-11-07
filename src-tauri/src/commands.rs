use crate::core::batch::{self, BatchItem};
use crate::core::exif;
use crate::core::geo::PlaceDb;
use crate::core::model::WatermarkConfig;
use crate::core::pipeline::{self, ResolvedSettings};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{imageops, RgbaImage};
use serde::Serialize;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

pub struct AppState {
    pub place_db: PlaceDb,
    pub cancel: Mutex<Option<Arc<AtomicBool>>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoMeta {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub datetime: Option<String>,
    pub place: Option<String>,
    pub orientation: u16,
    pub error: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub done: usize,
    pub total: usize,
    pub current: String,
    pub failed: usize,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DemoConfig {
    pub dir: Option<String>,
    pub view: Option<String>,
    pub style: Option<String>,
    pub readability: Option<String>,
    pub date: Option<String>,
    pub place: Option<String>,
}

/// 仅用于截图/演示：所有值来自环境变量，正常启动时全部为空。
#[tauri::command]
pub fn demo_config() -> DemoConfig {
    let read = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    DemoConfig {
        dir: read("DARKROOM_DEMO_DIR"),
        view: read("DARKROOM_DEMO_VIEW"),
        style: read("DARKROOM_DEMO_STYLE"),
        readability: read("DARKROOM_DEMO_READABILITY"),
        date: read("DARKROOM_DEMO_DATE"),
        place: read("DARKROOM_DEMO_PLACE"),
    }
}

fn encode_jpeg_data_url(img: &RgbaImage, quality: u8) -> Result<String, String> {
    let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buf);
        let mut enc =
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        enc.encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(format!(
        "data:image/jpeg;base64,{}",
        STANDARD.encode(&buf)
    ))
}

#[tauri::command]
pub fn pick_folder() -> Option<String> {
    rfd::FileDialog::new()
        .pick_folder()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn list_photos(dir: String) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(&dir).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        if matches!(
            ext.as_deref(),
            Some("jpg" | "jpeg" | "png" | "heic" | "heif")
        ) {
            out.push(entry.path().to_string_lossy().to_string());
        }
    }
    out.sort();
    Ok(out)
}

#[tauri::command]
pub fn read_photo_meta(state: tauri::State<AppState>, path: String) -> PhotoMeta {
    let p = PathBuf::from(&path);
    let bytes = std::fs::read(&p);
    let info = match &bytes {
        Ok(b) => exif::parse(b),
        Err(_) => exif::ExifInfo::default(),
    };
    let place = info
        .gps
        .and_then(|(lat, lng)| state.place_db.nearest(lat, lng).map(|e| e.name.clone()));
    let dims = image::image_dimensions(&p)
        .or_else(|_| crate::core::heic::decode(&p).map(|i| (i.width(), i.height())));
    let (width, height) = dims.unwrap_or((0, 0));
    let error = match bytes {
        Err(e) => Some(e.to_string()),
        Ok(_) => None,
    };
    PhotoMeta {
        path,
        width,
        height,
        datetime: info.datetime,
        place,
        orientation: info.orientation,
        error,
    }
}

#[tauri::command]
pub fn make_thumb(path: String, max: u32, orientation: u16) -> Result<String, String> {
    let img = pipeline::load_oriented(Path::new(&path), orientation)?;
    let thumb = imageops::thumbnail(&img, max, max);
    encode_jpeg_data_url(&thumb, 78)
}

#[tauri::command]
pub fn make_proxy(path: String, orientation: u16, max: u32) -> Result<String, String> {
    let img = pipeline::load_oriented(Path::new(&path), orientation)?;
    let longest = img.width().max(img.height()).max(1);
    let scale = (max as f32 / longest as f32).min(1.0);
    let w = ((img.width() as f32 * scale).round() as u32).max(1);
    let h = ((img.height() as f32 * scale).round() as u32).max(1);
    let small = imageops::resize(&img, w, h, imageops::FilterType::Triangle);
    encode_jpeg_data_url(&small, 86)
}

#[tauri::command]
pub fn render_preview(
    path: String,
    settings: ResolvedSettings,
    date: String,
    place: String,
    orientation: u16,
    max_dim: u32,
    focus: (f32, f32),
) -> Result<String, String> {
    let img = pipeline::compose(Path::new(&path), &settings, &date, &place, orientation, focus)?;
    let longest = img.width().max(img.height()).max(1);
    let scale = (max_dim as f32 / longest as f32).min(1.0);
    let w = ((img.width() as f32 * scale).round() as u32).max(1);
    let h = ((img.height() as f32 * scale).round() as u32).max(1);
    let small = imageops::resize(&img, w, h, imageops::FilterType::Triangle);
    encode_jpeg_data_url(&small, 84)
}

#[tauri::command]
pub fn build_watermark_svg(
    w: u32,
    h: u32,
    cfg: WatermarkConfig,
    date: String,
    place: String,
) -> String {
    crate::core::watermark::build_svg(w, h, &cfg, &date, &place)
}

#[tauri::command]
pub fn cancel(state: tauri::State<AppState>) {
    if let Some(flag) = state.cancel.lock().unwrap().as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub async fn export_batch(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    items: Vec<BatchItem>,
    settings: ResolvedSettings,
    out_dir: String,
) -> Result<Vec<String>, String> {
    let cancel = Arc::new(AtomicBool::new(false));
    *state.cancel.lock().unwrap() = Some(cancel.clone());
    let out = PathBuf::from(out_dir);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;

    let handle = app.clone();
    let results = tauri::async_runtime::spawn_blocking(move || {
        batch::run(&items, &settings, &out, &cancel, move |p| {
            let _ = handle.emit(
                "export-progress",
                ExportProgress {
                    done: p.done,
                    total: p.total,
                    current: p.current,
                    failed: p.failed,
                },
            );
        })
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(results.into_iter().filter_map(|r| r.err()).collect())
}
