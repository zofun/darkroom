use std::path::{Path, PathBuf};

pub fn filename(base: &str, size_label: &str, pattern: &str, ext: &str) -> String {
    let name = pattern.replace("{name}", base).replace("{size}", size_label);
    format!("{name}.{ext}")
}

pub fn unique_path(dir: &Path, desired: &Path) -> PathBuf {
    let stem = desired
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let ext = desired
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
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
    let tmp = dir.join(format!(
        ".tmp-{}",
        final_path.file_name().unwrap().to_string_lossy()
    ));
    write(&tmp)?;
    std::fs::rename(&tmp, final_path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("原子改名失败: {e}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn pattern_replaces_name_and_size() {
        assert_eq!(
            filename("IMG_0001", "6寸", "{name}_{size}", "jpg"),
            "IMG_0001_6寸.jpg"
        );
    }

    #[test]
    fn unique_path_appends_suffix() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.jpg"), b"x").unwrap();
        let p = unique_path(dir.path(), Path::new("a.jpg"));
        assert_eq!(p.file_name().unwrap(), "a_1.jpg");
    }
}
