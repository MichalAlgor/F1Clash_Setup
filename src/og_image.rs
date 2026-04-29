//! Generates a 1734×907 PNG preview card for share links (og:image).
//!
//! Loads `static/og-template.png` as the background, then composites a
//! transparent SVG overlay (text + card shapes) on top via resvg/tiny-skia.

use resvg::{tiny_skia, usvg};

use crate::data::StatPriorities;
use crate::models::part::PartCategory;
use crate::routes::share::{DriverSnapshot, PartSnapshot};

const FONT_DATA: &[u8] = include_bytes!("../static/fonts/roboto.ttf");
const TEMPLATE_DATA: &[u8] = include_bytes!("../static/og-template.png");
const W: u32 = 1734;
const H: u32 = 907;

// ── Colour helpers ────────────────────────────────────────────────────────────

fn rarity_hex(rarity: &str) -> &'static str {
    match rarity {
        "Common" => "#4a90d9",
        "Rare" => "#ed7d31",
        "Epic" => "#b46dd8",
        "Legendary" => "#ffd700",
        _ => "#aaaaaa",
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ── SVG helpers ───────────────────────────────────────────────────────────────

fn card_rect(x: i32, y: i32, w: i32, h: i32) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\" \
         rx=\"12\" fill=\"#080d1a\" fill-opacity=\"0.82\"/>"
    )
}

fn card_title(x: i32, y: i32, label: &str) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" fill=\"#4fc3f7\" font-size=\"18\" \
         font-weight=\"bold\" font-family=\"__FONT__\">{label}</text>"
    )
}

fn summary_row(lx: i32, rx: i32, y: i32, label: &str, value: &str) -> String {
    format!(
        "<text x=\"{lx}\" y=\"{y}\" fill=\"#8899aa\" font-size=\"15\" \
         font-family=\"__FONT__\">{label}</text>\
         <text x=\"{rx}\" y=\"{y}\" fill=\"#ffffff\" font-size=\"15\" \
         text-anchor=\"end\" font-family=\"__FONT__\">{value}</text>"
    )
}

// ── Overlay builder ───────────────────────────────────────────────────────────

fn build_overlay(
    name: &str,
    season: &str,
    priority_label: &str,
    parts: &[PartSnapshot],
    drivers: &[DriverSnapshot],
    parts_total: i64,
    drivers_total: i64,
    combined: i64,
    view_count: i32,
    created_at: &str,
) -> String {
    // Sort parts by canonical category order (PartCategory::all()).
    let mut sorted_parts = parts.to_vec();
    sorted_parts.sort_by_key(|p| {
        PartCategory::all()
            .iter()
            .position(|cat| cat.display_name() == p.category)
            .unwrap_or(usize::MAX)
    });
    let parts = sorted_parts.as_slice();

    let mut s = String::with_capacity(16_384);

    // Root SVG — transparent background (no fill rect)
    s.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{W}\" height=\"{H}\">"
    ));

    // ── Top section ───────────────────────────────────────────────────────────

    s.push_str(
        "<text x=\"60\" y=\"148\" fill=\"#4fc3f7\" font-size=\"20\" \
         font-weight=\"bold\" letter-spacing=\"2\" font-family=\"__FONT__\">F1 SETUP</text>",
    );

    // Setup name — truncate at 22 chars to avoid overflow into the car
    let name_display = if name.chars().count() > 22 {
        format!("{}…", name.chars().take(22).collect::<String>())
    } else {
        name.to_string()
    };
    s.push_str(&format!(
        "<text x=\"60\" y=\"232\" fill=\"#ffffff\" font-size=\"64\" \
         font-weight=\"bold\" font-family=\"__FONT__\">{}</text>",
        xml_escape(&name_display)
    ));

    // Season badge
    s.push_str("<rect x=\"60\" y=\"250\" width=\"172\" height=\"38\" rx=\"19\" fill=\"#1e2d5a\"/>");
    s.push_str(&format!(
        "<text x=\"146\" y=\"275\" fill=\"#ffffff\" font-size=\"20\" \
         text-anchor=\"middle\" font-family=\"__FONT__\">Season {}</text>",
        xml_escape(season)
    ));

    // Tagline + priority
    s.push_str(
        "<text x=\"60\" y=\"330\" fill=\"#8899aa\" font-size=\"21\" \
         font-family=\"__FONT__\">Optimized. Tested. Shared to win.</text>",
    );
    if !priority_label.is_empty() {
        s.push_str(&format!(
            "<text x=\"60\" y=\"360\" fill=\"#4fc3f7\" font-size=\"17\" \
             font-family=\"__FONT__\">{}</text>",
            xml_escape(priority_label)
        ));
    }

    // ── Stats bar ─────────────────────────────────────────────────────────────

    s.push_str(
        "<rect x=\"60\" y=\"378\" width=\"790\" height=\"88\" rx=\"12\" \
         fill=\"#080d1a\" fill-opacity=\"0.82\"/>",
    );

    // Dividers
    s.push_str(
        "<line x1=\"323\" y1=\"394\" x2=\"323\" y2=\"450\" stroke=\"#2a3a4a\" stroke-width=\"1\"/>",
    );
    s.push_str(
        "<line x1=\"587\" y1=\"394\" x2=\"587\" y2=\"450\" stroke=\"#2a3a4a\" stroke-width=\"1\"/>",
    );

    // Stat: Views
    s.push_str(&format!(
        "<text x=\"192\" y=\"419\" fill=\"#ffffff\" font-size=\"30\" \
         font-weight=\"bold\" text-anchor=\"middle\" font-family=\"__FONT__\">{view_count}</text>\
         <text x=\"192\" y=\"441\" fill=\"#8899aa\" font-size=\"13\" \
         text-anchor=\"middle\" font-family=\"__FONT__\">VIEWS</text>"
    ));

    // Stat: Total Score
    s.push_str(&format!(
        "<text x=\"455\" y=\"419\" fill=\"#ffd700\" font-size=\"30\" \
         font-weight=\"bold\" text-anchor=\"middle\" font-family=\"__FONT__\">{combined}</text>\
         <text x=\"455\" y=\"441\" fill=\"#8899aa\" font-size=\"13\" \
         text-anchor=\"middle\" font-family=\"__FONT__\">TOTAL SCORE</text>"
    ));

    // Stat: Parts Score
    s.push_str(&format!(
        "<text x=\"719\" y=\"419\" fill=\"#b46dd8\" font-size=\"30\" \
         font-weight=\"bold\" text-anchor=\"middle\" font-family=\"__FONT__\">{parts_total}</text>\
         <text x=\"719\" y=\"441\" fill=\"#8899aa\" font-size=\"13\" \
         text-anchor=\"middle\" font-family=\"__FONT__\">PARTS SCORE</text>"
    ));

    // ── Bottom cards ──────────────────────────────────────────────────────────

    let cy: i32 = 482; // card top y
    let ch: i32 = 370; // card height

    // Card 1 — Top Parts (x=60, w=258)
    let c1x = 60i32;
    let c1w = 258i32;
    s.push_str(&card_rect(c1x, cy, c1w, ch));
    s.push_str(&card_title(c1x + 24, cy + 38, "PARTS"));

    for (i, part) in parts.iter().take(7).enumerate() {
        let py = cy + 68 + i as i32 * 36;
        let color = rarity_hex(&part.rarity);
        let right_x = c1x + c1w - 16;
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" fill=\"#8899aa\" font-size=\"12\" \
             font-family=\"__FONT__\">{}</text>",
            c1x + 20,
            py,
            xml_escape(&part.category)
        ));
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" fill=\"{color}\" font-size=\"15\" \
             font-weight=\"bold\" font-family=\"__FONT__\">{}</text>",
            c1x + 20,
            py + 16,
            xml_escape(&part.part_name)
        ));
        s.push_str(&format!(
            "<text x=\"{right_x}\" y=\"{}\" fill=\"#4fc3f7\" font-size=\"17\" \
             font-weight=\"bold\" text-anchor=\"end\" font-family=\"__FONT__\">{}</text>",
            py + 16,
            part.total
        ));
    }

    // Parts total row
    let divider_y = cy + ch - 50;
    s.push_str(&format!(
        "<line x1=\"{}\" y1=\"{divider_y}\" x2=\"{}\" y2=\"{divider_y}\" \
         stroke=\"#2a3a4a\" stroke-width=\"1\"/>",
        c1x + 12,
        c1x + c1w - 12
    ));
    s.push_str(&format!(
        "<text x=\"{}\" y=\"{}\" fill=\"#8899aa\" font-size=\"13\" \
         font-weight=\"bold\" font-family=\"__FONT__\">TOTAL SCORE</text>",
        c1x + 20,
        cy + ch - 22
    ));
    s.push_str(&format!(
        "<text x=\"{}\" y=\"{}\" fill=\"#4fc3f7\" font-size=\"20\" \
         font-weight=\"bold\" text-anchor=\"end\" font-family=\"__FONT__\">{parts_total}</text>",
        c1x + c1w - 16,
        cy + ch - 22
    ));

    // Card 2 — Top Drivers (x=330, w=220)
    let c2x = 330i32;
    let c2w = 220i32;
    s.push_str(&card_rect(c2x, cy, c2w, ch));
    s.push_str(&card_title(c2x + 24, cy + 38, "DRIVERS"));

    if drivers.is_empty() {
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" fill=\"#8899aa\" font-size=\"15\" \
             font-family=\"__FONT__\">No drivers selected</text>",
            c2x + 20,
            cy + 80
        ));
    } else {
        for (i, d) in drivers.iter().take(2).enumerate() {
            let dy = cy + 76 + i as i32 * 76;
            let d_color = rarity_hex(&d.rarity);
            let right_x = c2x + c2w - 16;
            s.push_str(&format!(
                "<text x=\"{}\" y=\"{dy}\" fill=\"{d_color}\" font-size=\"17\" \
                 font-weight=\"bold\" font-family=\"__FONT__\">{}</text>",
                c2x + 20,
                xml_escape(&d.driver_name)
            ));
            s.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" fill=\"#8899aa\" font-size=\"13\" \
                 font-family=\"__FONT__\">({})</text>",
                c2x + 20,
                dy + 18,
                xml_escape(&d.rarity)
            ));
            s.push_str(&format!(
                "<text x=\"{right_x}\" y=\"{}\" fill=\"#4fc3f7\" font-size=\"20\" \
                 font-weight=\"bold\" text-anchor=\"end\" font-family=\"__FONT__\">{}</text>",
                dy + 10,
                d.total
            ));
        }

        let d_divider_y = cy + ch - 50;
        s.push_str(&format!(
            "<line x1=\"{}\" y1=\"{d_divider_y}\" x2=\"{}\" y2=\"{d_divider_y}\" \
             stroke=\"#2a3a4a\" stroke-width=\"1\"/>",
            c2x + 12,
            c2x + c2w - 12
        ));
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" fill=\"#8899aa\" font-size=\"13\" \
             font-weight=\"bold\" font-family=\"__FONT__\">DRIVERS SCORE</text>",
            c2x + 20,
            cy + ch - 22
        ));
        s.push_str(&format!(
            "<text x=\"{}\" y=\"{}\" fill=\"#ffd700\" font-size=\"20\" \
             font-weight=\"bold\" text-anchor=\"end\" font-family=\"__FONT__\">{drivers_total}</text>",
            c2x + c2w - 16,
            cy + ch - 22
        ));
    }

    // Card 3 — Setup Summary (x=562, w=280)
    let c3x = 562i32;
    let c3w = 280i32;
    s.push_str(&card_rect(c3x, cy, c3w, ch));
    s.push_str(&card_title(c3x + 24, cy + 38, "SETUP SUMMARY"));

    let row_lx = c3x + 20;
    let row_rx = c3x + c3w - 16;
    let row1_y = cy + 80;
    let row_step = 48i32;

    // Truncate name for summary
    let name_short = if name.chars().count() > 14 {
        format!("{}…", name.chars().take(14).collect::<String>())
    } else {
        name.to_string()
    };

    s.push_str(&summary_row(
        row_lx,
        row_rx,
        row1_y,
        "Setup Name",
        &xml_escape(&name_short),
    ));
    s.push_str(&summary_row(
        row_lx,
        row_rx,
        row1_y + row_step,
        "Season",
        &xml_escape(season),
    ));
    s.push_str(&summary_row(
        row_lx,
        row_rx,
        row1_y + row_step * 2,
        "Created",
        &xml_escape(created_at),
    ));
    s.push_str(&summary_row(
        row_lx,
        row_rx,
        row1_y + row_step * 3,
        "Views",
        &view_count.to_string(),
    ));
    s.push_str("</svg>");
    s
}

// ── Public render entry point ─────────────────────────────────────────────────

/// Composites dynamic share data onto the og-template.png background.
/// Returns empty `Vec` on failure (logged via `tracing`).
pub fn render_og_image(
    name: &str,
    season: &str,
    priorities: &StatPriorities,
    parts: &[PartSnapshot],
    drivers: &[DriverSnapshot],
    parts_total: i64,
    drivers_total: i64,
    view_count: i32,
    created_at: &str,
) -> Vec<u8> {
    let combined = parts_total + drivers_total;
    let priority_label = {
        let labels = priorities.labels();
        if labels.is_empty() {
            String::new()
        } else {
            labels.join(" + ")
        }
    };

    let svg_raw = build_overlay(
        name,
        season,
        &priority_label,
        parts,
        drivers,
        parts_total,
        drivers_total,
        combined,
        view_count,
        created_at,
    );

    // ── Font discovery ────────────────────────────────────────────────────────
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_font_data(FONT_DATA.to_vec());
    let font_family = fontdb
        .faces()
        .next()
        .and_then(|f| f.families.first())
        .map(|(n, _)| n.clone())
        .unwrap_or_else(|| "sans-serif".to_string());

    let svg = svg_raw.replace("__FONT__", &font_family);

    let mut opt = usvg::Options::default();
    opt.font_family = font_family;
    opt.fontdb = std::sync::Arc::new(fontdb);

    let tree = match usvg::Tree::from_str(&svg, &opt) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("og_image: SVG parse error: {e}");
            return Vec::new();
        }
    };

    // ── Load template as background pixmap ───────────────────────────────────
    let mut pixmap = match tiny_skia::Pixmap::decode_png(TEMPLATE_DATA) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("og_image: failed to decode template PNG: {e}");
            return Vec::new();
        }
    };

    // Render SVG overlay onto template (transparent areas leave template intact)
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );

    match pixmap.encode_png() {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::error!("og_image: PNG encode error: {e}");
            Vec::new()
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::StatPriorities;
    use crate::routes::share::{DriverSnapshot, PartSnapshot};

    fn sample_parts() -> Vec<PartSnapshot> {
        vec![
            PartSnapshot {
                category: "Engine".to_string(),
                part_name: "Engine Tempest".to_string(),
                level: 8,
                rarity: "Epic".to_string(),
                speed: 80,
                cornering: 60,
                power_unit: 70,
                qualifying: 50,
                pit_stop_time: 2.10,
                additional_stat_value: 0,
                total: 216,
            },
            PartSnapshot {
                category: "Front Wing".to_string(),
                part_name: "Front Wing Zephyr".to_string(),
                level: 7,
                rarity: "Rare".to_string(),
                speed: 65,
                cornering: 72,
                power_unit: 55,
                qualifying: 60,
                pit_stop_time: 2.30,
                additional_stat_value: 0,
                total: 225,
            },
            PartSnapshot {
                category: "Rear Wing".to_string(),
                part_name: "Rear Wing Wobble".to_string(),
                level: 6,
                rarity: "Common".to_string(),
                speed: 70,
                cornering: 45,
                power_unit: 60,
                qualifying: 55,
                pit_stop_time: 2.50,
                additional_stat_value: 0,
                total: 219,
            },
            PartSnapshot {
                category: "Suspension".to_string(),
                part_name: "Suspension Jumpstart".to_string(),
                level: 5,
                rarity: "Rare".to_string(),
                speed: 55,
                cornering: 80,
                power_unit: 50,
                qualifying: 65,
                pit_stop_time: 2.20,
                additional_stat_value: 0,
                total: 208,
            },
            PartSnapshot {
                category: "Brakes".to_string(),
                part_name: "Brakes Flow 2K".to_string(),
                level: 7,
                rarity: "Common".to_string(),
                speed: 60,
                cornering: 55,
                power_unit: 45,
                qualifying: 50,
                pit_stop_time: 2.40,
                additional_stat_value: 0,
                total: 204,
            },
            PartSnapshot {
                category: "Gearbox".to_string(),
                part_name: "Gearbox Stratos".to_string(),
                level: 6,
                rarity: "Rare".to_string(),
                speed: 65,
                cornering: 60,
                power_unit: 55,
                qualifying: 58,
                pit_stop_time: 2.15,
                additional_stat_value: 0,
                total: 217,
            },
            PartSnapshot {
                category: "Battery".to_string(),
                part_name: "Battery Surge".to_string(),
                level: 5,
                rarity: "Legendary".to_string(),
                speed: 72,
                cornering: 65,
                power_unit: 80,
                qualifying: 70,
                pit_stop_time: 2.05,
                additional_stat_value: 0,
                total: 231,
            },
        ]
    }

    fn sample_drivers() -> Vec<DriverSnapshot> {
        vec![
            DriverSnapshot {
                driver_name: "Arvid Lindblad".to_string(),
                rarity: "Epic".to_string(),
                level: 8,
                overtaking: 30,
                defending: 28,
                qualifying: 25,
                race_start: 24,
                tyre_management: 24,
                total: 131,
            },
            DriverSnapshot {
                driver_name: "Bruce Mclaren".to_string(),
                rarity: "Legendary".to_string(),
                level: 7,
                overtaking: 32,
                defending: 27,
                qualifying: 26,
                race_start: 25,
                tyre_management: 25,
                total: 135,
            },
        ]
    }

    /// Prints the discovered font family name.
    ///   cargo test og_image::tests::font_family_name -- --nocapture
    #[test]
    fn font_family_name() {
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_font_data(FONT_DATA.to_vec());
        let faces: Vec<_> = fontdb.faces().collect();
        println!("Loaded {} font face(s)", faces.len());
        for face in &faces {
            println!("  families: {:?}", face.families);
        }
        assert!(!faces.is_empty(), "no font faces loaded from bundled TTF");
    }

    /// Generates a PNG and writes it to /tmp/og_test.png.
    ///   cargo test og_image::tests::write_png -- --nocapture
    ///   open /tmp/og_test.png
    #[test]
    fn write_png() {
        let parts = sample_parts();
        let drivers = sample_drivers();
        let priorities = StatPriorities {
            speed: true,
            cornering: true,
            power_unit: false,
            qualifying: false,
        };

        let bytes = render_og_image(
            "Shared Setup",
            "2026",
            &priorities,
            &parts,
            &drivers,
            239,
            266,
            6,
            "Apr 25, 2025",
        );

        assert!(
            !bytes.is_empty(),
            "render_og_image returned empty bytes — check tracing logs"
        );
        assert_eq!(
            &bytes[..8],
            b"\x89PNG\r\n\x1a\n",
            "output is not a valid PNG"
        );

        std::fs::write("/tmp/og_test.png", &bytes).expect("failed to write /tmp/og_test.png");
        println!(
            "PNG written ({} bytes) — open /tmp/og_test.png",
            bytes.len()
        );
    }
}
