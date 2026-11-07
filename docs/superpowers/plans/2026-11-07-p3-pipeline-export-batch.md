# Phase 1c · Rust 核心（管线 / 导出 / 批处理）

> 承接 `2026-11-07-p2-watermark-heic.md`。

## Task 8: 单张合成管线

**Files:**
- Create: `src-tauri/src/core/pipeline.rs`
- Modify: `src-tauri/src/core/mod.rs`
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加依赖**

`src-tauri/Cargo.toml` 的 `[dependencies]`：
```toml
image = { version = "0.25", default-features = false, features = ["jpeg", "png", "rayon"] }
resvg = "0.45"
usvg = "0.45"
rayon = "1.10"
walkdir = "2"
tempfile = "3"
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 2: 写失败测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{CropMode, PrintSize, WatermarkConfig};
    use image::{Rgba, RgbaImage};

    fn settings() -> ResolvedSettings {
        ResolvedSettings {
            size: PrintSize { width_mm: 101.6, height_mm: 152.4, dpi: 300 },
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
    fn output_has_target_dimensions() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.png");
        let mut img = RgbaImage::new(3000, 2000);
        for p in img.pixels_mut() { *p = Rgba([120, 150, 100, 255]); }
        img.save(&input).unwrap();

        let output = dir.path().join("out.png");
        process(&input, &output, &settings(), "2024 · 六月", "杭州市", 1).unwrap();

        let out = image::open(&output).unwrap().to_rgba8();
        assert_eq!(out.dimensions(), (1200, 1800));
    }

    #[test]
    fn fit_mode_paints_border_color() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("in.png");
        let mut img = RgbaImage::new(3000, 2000);
        for p in img.pixels_mut() { *p = Rgba([0, 0, 0, 255]); }
        img.save(&input).unwrap();

        let mut s = settings();
        s.crop_mode = CropMode::Fit;
        s.border_enabled = true;
        s.border_color = [255, 0, 0];
        let output = dir.path().join("out.png");
        process(&input, &output, &s, "", "", 1).unwrap();

        let out = image::open(&output).unwrap().to_rgba8();
        let top = out.get_pixel(600, 300);
        assert_eq!(top.0[0], 255);
        assert_eq!(top.0[1], 0);
    }
}
```

- [ ] **Step 3: 运行确认失败**

Run:
```powershell
cargo test --manifest-path src-tauri/Cargo.toml pipeline::
```
Expected: 编译错误。

- [ ] **Step 4: 实现**

```rust
use crate::core::model::{CropMode, PrintSize, WatermarkConfig};
use crate::core::{geometry, watermark};
use image::{imageops, Rgba, RgbaImage};
use std::path::Path;

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

fn render_overlay(w: u32, h: u32, cfg: &WatermarkConfig, date: &str, place: &str) -> Option<RgbaImage> {
    let svg = watermark::build_svg(w, h, cfg, date, place);
    let mut opt = usvg::Options::default();
    const FONT: &[u8] = include_bytes!("../../assets/fonts/SourceHanSerifSC.otf");
    opt.fontdb_mut().load_font_data(FONT.to_vec());
    let tree = usvg::Tree::from_str(&svg, &opt).ok()?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h)?;
    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

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

pub fn process(
    input: &Path,
    output: &Path,
    s: &ResolvedSettings,
    date: &str,
    place: &str,
    orientation: u16,
) -> Result<(), String> {
    let source = match input.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) {
        Some(ext) if ext == "heic" || ext == "heif" => crate::core::heic::decode(input)?,
        _ => image::open(input).map_err(|e| format!("解码失败: {e}"))?.to_rgba8(),
    };
    let source = apply_orientation(source, orientation);
    let (tw, th) = s.size.pixels();

    let canvas: RgbaImage = match s.crop_mode {
        CropMode::Fill => {
            let (x, y, cw, ch) = geometry::fill_rect(source.width(), source.height(), tw, th, (0.5, 0.5));
            let cropped = imageops::crop_imm(&source, x, y, cw, ch).to_image();
            imageops::resize(&cropped, tw, th, imageops::FilterType::Lanczos3)
        }
        CropMode::Fit => {
            let mut canvas = RgbaImage::from_pixel(tw, th, Rgba([
                s.border_color[0], s.border_color[1], s.border_color[2], 255,
            ]));
            let (fx, fy, fw, fh) = geometry::fit_rect(source.width(), source.height(), tw, th);
            let resized = imageops::resize(&source, fw, fh, imageops::FilterType::Lanczos3);
            imageops::overlay(&mut canvas, &resized, fx as i64, fy as i64);
            canvas
        }
    };

    let mut final_img = canvas;
    if let Some(overlay) = render_overlay(tw, th, &s.watermark, date, place) {
        imageops::overlay(&mut final_img, &overlay, 0, 0);
    }

    if s.format == "png" {
        final_img.save(output).map_err(|e| format!("编码失败: {e}"))?;
    } else {
        let file = std::fs::File::create(output).map_err(|e| format!("创建文件失败: {e}"))?;
        let mut writer = std::io::BufWriter::new(file);
        let rgb = image::DynamicImage::ImageRgba8(final_img).to_rgb8();
        let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, s.quality);
        enc.encode(rgb.as_raw(), rgb.width(), rgb.height(), image::ExtendedColorType::Rgb8)
            .map_err(|e| format!("JPEG 编码失败: {e}"))?;
    }
    Ok(())
}
```

- [ ] **Step 5: 放置字体文件**

把 OFL 许可的思源宋体放到 `src-tauri/assets/fonts/SourceHanSerifSC.otf`。缺失时 `include_bytes!` 会导致编译失败，必须先放入。

- [ ] **Step 6: 运行确认通过**

Run:
```powershell
cargo test --manifest-path src-tauri/Cargo.toml pipeline::
```
Expected: 2 个测试 PASS。

- [ ] **Step 7: 注册并提交**

`core/mod.rs` 追加 `pub mod pipeline;`。

```powershell
git add src-tauri/Cargo.toml src-tauri/src/core src-tauri/assets/fonts
git commit -m "feat(core): compose single photo with watermark overlay"
```

---

## Task 9: 导出命名与原子写入

**Files:**
- Create: `src-tauri/src/core/export.rs`
- Modify: `src-tauri/src/core/mod.rs`

- [ ] **Step 1: 写失败测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn pattern_replaces_name_and_size() {
        assert_eq!(filename("IMG_0001", "6寸", "{name}_{size}", "jpg"), "IMG_0001_6寸.jpg");
    }

    #[test]
    fn unique_path_appends_suffix() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.jpg"), b"x").unwrap();
        let p = unique_path(dir.path(), Path::new("a.jpg"));
        assert_eq!(p.file_name().unwrap(), "a_1.jpg");
    }
}
```

- [ ] **Step 2: 运行确认失败**

Run:
```powershell
cargo test --manifest-path src-tauri/Cargo.toml export::
```
Expected: 编译错误。

- [ ] **Step 3: 实现**

```rust
use std::path::{Path, PathBuf};

pub fn filename(base: &str, size_label: &str, pattern: &str, ext: &str) -> String {
    let name = pattern.replace("{name}", base).replace("{size}", size_label);
    format!("{name}.{ext}")
}

pub fn unique_path(dir: &Path, desired: &Path) -> PathBuf {
    let stem = desired.file_stem().unwrap().to_string_lossy().to_string();
    let ext = desired.extension().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
    let mut candidate = dir.join(desired.file_name().unwrap());
    let mut i = 1;
    while candidate.exists() {
        let name = if ext.is_empty() {
            format!("{stem}_{i}")
        } else {
            format!("{stem}_{i}.{ext}")
        };
        candidate = dir.join(name);
        i += 1;
    }
    candidate
}

pub fn atomic_write<F>(final_path: &Path, write: F) -> Result<(), String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    let dir = final_path.parent().ok_or("无父目录")?;
    let tmp = dir.join(format!(".tmp-{}", final_path.file_name().unwrap().to_string_lossy()));
    write(&tmp)?;
    std::fs::rename(&tmp, final_path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("原子改名失败: {e}")
    })
}
```

- [ ] **Step 4: 运行确认通过**

Run:
```powershell
cargo test --manifest-path src-tauri/Cargo.toml export::
```
Expected: 2 个测试 PASS。

- [ ] **Step 5: 注册并提交**

`core/mod.rs` 追加 `pub mod export;`。

```powershell
git add src-tauri/src/core
git commit -m "feat(core): export filename pattern and atomic write"
```

---

## Task 10: 并行批处理与进度/取消

**Files:**
- Create: `src-tauri/src/core/batch.rs`
- Modify: `src-tauri/src/core/mod.rs`

- [ ] **Step 1: 写失败测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{CropMode, PrintSize, WatermarkConfig};
    use image::{Rgba, RgbaImage};
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;

    fn settings() -> ResolvedSettings {
        ResolvedSettings {
            size: PrintSize { width_mm: 101.6, height_mm: 152.4, dpi: 300 },
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
    fn batch_processes_all_and_reports_progress() {
        let dir = tempfile::tempdir().unwrap();
        let mut items = Vec::new();
        for i in 0..3 {
            let p = dir.path().join(format!("in{i}.png"));
            let mut img = RgbaImage::new(600, 400);
            for px in img.pixels_mut() { *px = Rgba([10, 20, 30, 255]); }
            img.save(&p).unwrap();
            items.push(BatchItem {
                input: p,
                date: "2024".into(),
                place: "杭州市".into(),
                orientation: 1,
                size_label: "6寸".into(),
            });
        }
        let out = dir.path().join("out");
        std::fs::create_dir_all(&out).unwrap();

        let done = Mutex::new(Vec::new());
        let cancel = AtomicBool::new(false);
        let results = run(&items, &settings(), &out, &cancel, |p| {
            done.lock().unwrap().push(p.done);
        });

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
        assert_eq!(*done.lock().unwrap().last().unwrap(), 3);
    }
}
```

- [ ] **Step 2: 运行确认失败**

Run:
```powershell
cargo test --manifest-path src-tauri/Cargo.toml batch::
```
Expected: 编译错误。

- [ ] **Step 3: 实现**

```rust
use crate::core::export;
use crate::core::pipeline::{self, ResolvedSettings};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchItem {
    pub input: PathBuf,
    pub date: String,
    pub place: String,
    pub orientation: u16,
    pub size_label: String,
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub current: String,
    pub failed: usize,
}

pub fn run<F>(
    items: &[BatchItem],
    settings: &ResolvedSettings,
    out_dir: &Path,
    cancel: &AtomicBool,
    on_progress: F,
) -> Vec<Result<PathBuf, String>>
where
    F: Fn(Progress) + Sync,
{
    let total = items.len();
    let done = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    let progress_lock = Mutex::new(());

    items
        .par_iter()
        .map(|item| {
            if cancel.load(Ordering::Relaxed) {
                return Err("已取消".to_string());
            }
            let base = item
                .input
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "photo".into());
            let ext = if settings.format == "png" { "png" } else { "jpg" };
            let name = export::filename(&base, &item.size_label, "{name}_{size}", ext);
            let target = export::unique_path(out_dir, Path::new(&name));
            let result = pipeline::process(
                &item.input,
                &target,
                settings,
                &item.date,
                &item.place,
                item.orientation,
            );
            if result.is_err() {
                failed.fetch_add(1, Ordering::Relaxed);
            }
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            {
                let _guard = progress_lock.lock().unwrap();
                on_progress(Progress {
                    done: n,
                    total,
                    current: base,
                    failed: failed.load(Ordering::Relaxed),
                });
            }
            result.map(|_| target)
        })
        .collect()
}
```

- [ ] **Step 4: 运行确认通过**

Run:
```powershell
cargo test --manifest-path src-tauri/Cargo.toml batch::
```
Expected: 1 个测试 PASS。

- [ ] **Step 5: 注册并提交**

`core/mod.rs` 追加 `pub mod batch;`。

```powershell
git add src-tauri/src/core
git commit -m "feat(core): parallel batch processing with progress and cancel"
```
