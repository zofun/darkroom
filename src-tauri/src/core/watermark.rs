use crate::core::model::{WatermarkConfig, WatermarkStyle};
use std::fmt::Write;

fn color(cfg: &WatermarkConfig) -> &'static str {
    match cfg.color_token.as_str() {
        "gold" => "#c8a96a",
        "paper" => "#e8e2d6",
        "ink" => "#1c1a17",
        _ => "#f4e8ca",
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
}

pub fn build_svg(w: u32, h: u32, cfg: &WatermarkConfig, date: &str, place: &str) -> String {
    let fw = w as f64;
    let fh = h as f64;
    let unit = fw.min(fh);
    let scale = cfg.scale.max(0.1) as f64;
    let date_size = unit * 0.045 * scale;
    let place_size = unit * 0.026 * scale;
    let c = color(cfg);
    let font = &cfg.font;
    let d = if cfg.show_date {
        esc(date)
    } else {
        String::new()
    };
    let p = if cfg.show_location {
        esc(place)
    } else {
        String::new()
    };

    let mut body = String::new();
    match cfg.style {
        WatermarkStyle::Engraved => {
            let x = fw * 0.94;
            let y = fh * 0.90;
            let _ = write!(
                body,
                r#"<g class='wm-engraved' text-anchor='end'>
                <text x='{x}' y='{y}' font-family='{font}' font-size='{date_size}' fill='{c}'>{d}</text>
                <line x1='{lx}' y1='{ly}' x2='{x}' y2='{ly}' stroke='{c}' stroke-width='{sw}'/>
                <text x='{x}' y='{py}' font-family='{font}' font-size='{place_size}' letter-spacing='{ls}' fill='{c}'>{p}</text>
                </g>"#,
                lx = x - fw * 0.06,
                ly = y + date_size * 0.55,
                sw = (unit * 0.0015).max(1.0),
                py = y + date_size * 0.55 + place_size * 1.4,
                ls = place_size * 0.28,
            );
        }
        WatermarkStyle::Siderail => {
            let band = fw * 0.06;
            let rx = fw - band;
            let _ = write!(
                body,
                r#"<g class='wm-siderail'>
                <rect x='{rx}' y='0' width='{band}' height='{fh}' fill='rgba(10,10,12,0.55)'/>
                <line x1='{rx}' y1='0' x2='{rx}' y2='{fh}' stroke='{c}' stroke-width='{sw}'/>
                <text x='{tx}' y='{ty}' transform='rotate(-90 {tx} {ty})' font-family='{font}' font-size='{date_size}' letter-spacing='{ls}' fill='{c}'>{d} {p}</text>
                </g>"#,
                sw = (unit * 0.0015).max(1.0),
                tx = fw - band * 0.65,
                ty = fh * 0.92,
                ls = place_size * 0.2,
            );
        }
        WatermarkStyle::Cornernote => {
            let x = fw * 0.05;
            let y = fh * 0.94;
            let _ = write!(
                body,
                r#"<g class='wm-cornernote'>
                <circle cx='{dotx}' cy='{doty}' r='{r}' fill='{c}'/>
                <text x='{tx}' y='{ty}' font-family='{font}' font-size='{ds}' font-weight='600' fill='{c}'>{d}</text>
                <text x='{tx}' y='{py}' font-family='{font}' font-size='{ps}' letter-spacing='{ls}' fill='{c}'>{p}</text>
                </g>"#,
                dotx = x + unit * 0.006,
                doty = y - date_size * 0.3,
                r = unit * 0.004,
                tx = x + unit * 0.02,
                ty = y,
                ds = date_size,
                py = y + place_size * 1.5,
                ps = place_size,
                ls = place_size * 0.16,
            );
        }
        WatermarkStyle::Footerband => {
            let band = fh * 0.09;
            let _ = write!(
                body,
                r#"<g class='wm-footerband'>
                <defs><linearGradient id='fade' x1='0' y1='1' x2='0' y2='0'>
                <stop offset='0' stop-color='#08080a' stop-opacity='0.72'/>
                <stop offset='1' stop-color='#08080a' stop-opacity='0'/>
                </linearGradient></defs>
                <rect x='0' y='{ry}' width='{fw}' height='{band}' fill='url(#fade)'/>
                <text x='{lx}' y='{ty}' font-family='{font}' font-size='{ds}' fill='{c}'>{d}</text>
                <text x='{rx}' y='{ty}' text-anchor='end' font-family='{font}' font-size='{ps}' letter-spacing='{ls}' fill='{c}'>{p}</text>
                </g>"#,
                ry = fh - band,
                lx = fw * 0.05,
                rx = fw * 0.95,
                ty = fh - band * 0.35,
                ds = date_size * 0.9,
                ps = place_size,
                ls = place_size * 0.2,
            );
        }
    }

    format!(
        r#"<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='0 0 {w} {h}'>{body}</svg>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(style: WatermarkStyle) -> WatermarkConfig {
        WatermarkConfig {
            style,
            ..Default::default()
        }
    }

    #[test]
    fn svg_has_canvas_and_text() {
        let svg = build_svg(
            1200,
            1800,
            &cfg(WatermarkStyle::Cornernote),
            "2024 · 六月",
            "杭州市",
        );
        assert!(svg.contains("width='1200'"));
        assert!(svg.contains("height='1800'"));
        assert!(svg.contains("2024 · 六月"));
        assert!(svg.contains("杭州市"));
    }

    #[test]
    fn each_style_is_marked() {
        let pairs = [
            (WatermarkStyle::Engraved, "wm-engraved"),
            (WatermarkStyle::Siderail, "wm-siderail"),
            (WatermarkStyle::Cornernote, "wm-cornernote"),
            (WatermarkStyle::Footerband, "wm-footerband"),
        ];
        for (s, token) in pairs {
            let svg = build_svg(1200, 1800, &cfg(s), "d", "l");
            assert!(svg.contains(token), "missing {token}");
        }
    }

    #[test]
    fn hidden_fields_omitted() {
        let mut c = cfg(WatermarkStyle::Cornernote);
        c.show_date = false;
        let svg = build_svg(1200, 1800, &c, "SECRET_DATE", "杭州市");
        assert!(!svg.contains("SECRET_DATE"));
        assert!(svg.contains("杭州市"));
    }
}
