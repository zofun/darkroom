/// 返回源图裁切窗口 (x, y, w, h)。focus 0..1 控制裁切偏向。
pub fn fill_rect(
    src_w: u32,
    src_h: u32,
    target_w: u32,
    target_h: u32,
    focus: (f32, f32),
) -> (u32, u32, u32, u32) {
    let targ_ar = target_w as f32 / target_h as f32;
    let src_ar = src_w as f32 / src_h as f32;
    if src_ar > targ_ar {
        let new_w = ((src_h as f32 * targ_ar).round() as u32).clamp(1, src_w);
        let max_x = src_w - new_w;
        let x = (max_x as f32 * focus.0.clamp(0.0, 1.0)).round() as u32;
        (x, 0, new_w, src_h)
    } else {
        let new_h = ((src_w as f32 / targ_ar).round() as u32).clamp(1, src_h);
        let max_y = src_h - new_h;
        let y = (max_y as f32 * focus.1.clamp(0.0, 1.0)).round() as u32;
        (0, y, src_w, new_h)
    }
}

/// 等比缩放进 target，返回目标画布上的 (x, y, w, h)。
pub fn fit_rect(src_w: u32, src_h: u32, target_w: u32, target_h: u32) -> (u32, u32, u32, u32) {
    let scale = (target_w as f32 / src_w as f32).min(target_h as f32 / src_h as f32);
    let w = ((src_w as f32 * scale).round() as u32).clamp(1, target_w);
    let h = ((src_h as f32 * scale).round() as u32).clamp(1, target_h);
    ((target_w - w) / 2, (target_h - h) / 2, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_crops_wide_image() {
        let r = fill_rect(3000, 2000, 1200, 1800, (0.5, 0.5));
        assert_eq!(r.2, (2000.0_f64 * 1200.0 / 1800.0).round() as u32);
        assert_eq!(r.3, 2000);
        assert_eq!(r.0, ((3000 - r.2) as f64 / 2.0).round() as u32);
    }

    #[test]
    fn fill_crops_tall_image() {
        let r = fill_rect(2000, 3000, 1800, 1200, (0.5, 0.5));
        assert_eq!(r.2, 2000);
        assert_eq!(r.3, (2000.0_f64 * 1200.0 / 1800.0).round() as u32);
    }

    #[test]
    fn fit_keeps_aspect_inside_target() {
        let r = fit_rect(3000, 2000, 1200, 1800);
        assert_eq!(r.2, 1200);
        assert_eq!(r.3, 800);
        assert_eq!(r.0, 0);
        assert_eq!(r.1, (1800 - 800) / 2);
    }

    #[test]
    fn focus_moves_crop_window() {
        let left = fill_rect(3000, 2000, 1200, 1800, (0.0, 0.5));
        let right = fill_rect(3000, 2000, 1200, 1800, (1.0, 0.5));
        assert_eq!(left.0, 0);
        assert_eq!(right.0, 3000 - right.2);
    }
}
