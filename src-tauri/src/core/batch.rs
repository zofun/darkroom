use crate::core::export;
use crate::core::pipeline::{self, ResolvedSettings};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Clone, Serialize, Deserialize)]
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

    let ncpu = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let threads = (ncpu / 2).clamp(1, 4);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("build rayon pool");

    pool.install(|| {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{CropMode, PrintSize, WatermarkConfig};
    use crate::core::pipeline::ResolvedSettings;
    use image::{Rgba, RgbaImage};
    use std::sync::atomic::AtomicBool;
    use std::sync::Mutex;

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
    fn batch_processes_all_and_reports_progress() {
        let dir = tempfile::tempdir().unwrap();
        let mut items = Vec::new();
        for i in 0..3 {
            let p = dir.path().join(format!("in{i}.png"));
            let mut img = RgbaImage::new(600, 400);
            for px in img.pixels_mut() {
                *px = Rgba([10, 20, 30, 255]);
            }
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
