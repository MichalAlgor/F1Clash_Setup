use maud::{Markup, PreEscaped, html};
use serde_json::Value;

use crate::auth::AuthStatus;
use crate::data::StatPriorities;
use crate::drivers_data::DriverRarity;
use crate::models::setup::InventoryItem;
use crate::routes::share::{DriverSnapshot, PartSnapshot};

fn part_rarity_class(rarity: &str) -> &'static str {
    match rarity {
        "Common" => "rarity-common",
        "Rare" => "rarity-rare",
        "Epic" => "rarity-epic",
        "Legendary" => "rarity-legendary",
        _ => "",
    }
}

/// Shown after successfully creating a share — displays the URL with a copy button.
pub fn shared_page(hash: &str, name: &str, back_href: &str, auth: &AuthStatus) -> Markup {
    let share_url = format!("/share/{hash}");
    let back_label = if back_href.starts_with("/setups") {
        "← Back to Setups"
    } else {
        "← Back to Optimizer"
    };
    super::layout::page(
        "Setup Shared",
        auth,
        html! {
            hgroup {
                h1 { "Setup Shared!" }
                p { "Anyone with this link can view your setup." }
            }

            p { strong { (name) } }

            div class="share-url-row" {
                code id="share-url" class="share-url" { (share_url) }
                button id="copy-btn" class="outline" onclick="copyShareUrl()" { "Copy Link" }
            }

            p class="secondary" style="font-size:0.85rem" {
                "The link captures your setup's stats at this moment. "
                "If you change your inventory or levels later, the shared link is unaffected."
            }

            div style="display:flex;gap:0.75rem;flex-wrap:wrap;margin-top:1rem" {
                a href=(share_url) role="button" class="outline" { "View shared setup" }
                a href=(back_href) role="button" class="outline" { (back_label) }
            }

            script {
                (PreEscaped(r#"
function copyShareUrl() {
    var url = window.location.origin + document.getElementById('share-url').textContent;
    navigator.clipboard.writeText(url).then(function() {
        var btn = document.getElementById('copy-btn');
        btn.textContent = 'Copied!';
        setTimeout(function() { btn.textContent = 'Copy Link'; }, 2000);
    });
}
                "#))
            }
        },
    )
}

/// 404 page for unknown share hashes.
pub fn not_found_page(auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Setup Not Found",
        auth,
        html! {
            h1 { "Setup Not Found" }
            p { "This shared setup link is invalid or has expired." }
            a href="/optimizer" role="button" class="outline" { "Try the Optimizer" }
        },
    )
}

pub struct SharePage {
    pub _hash: String,
    pub name: String,
    pub season: String,
    pub priorities: StatPriorities,
    pub total_parts: Value,
    pub total_drivers: Value,
    pub view_count: i32,
}

/// Public view of a shared setup snapshot.
pub fn view_page(
    share_page: &SharePage,
    parts: &[PartSnapshot],
    drivers: &[DriverSnapshot],
    viewer_inventory: &[InventoryItem],
    auth: &AuthStatus,
) -> Markup {
    let priority_label = {
        let labels = share_page.priorities.labels();
        if labels.is_empty() {
            "Total performance".to_string()
        } else {
            labels.join(", ")
        }
    };
    let parts_total = share_page.total_parts["total"].as_i64().unwrap_or(0);
    let drivers_total = share_page.total_drivers["total"].as_i64().unwrap_or(0);

    let og_title = share_page.name.clone();
    let og_description = {
        let score = parts_total + drivers_total;
        if drivers_total > 0 {
            format!(
                "Season {} · {} · Score: {} ({} parts + {} drivers)",
                share_page.season, priority_label, score, parts_total, drivers_total
            )
        } else {
            format!(
                "Season {} · {} · Score: {}",
                share_page.season, priority_label, score
            )
        }
    };

    super::layout::page_with_og(
        &format!("Shared: {}", share_page.name),
        &og_title,
        &og_description,
        auth,
        html! {
            hgroup {
                h1 { "Shared Setup" }
                p {
                    strong { (share_page.name) }
                    " · Season " (share_page.season)
                    " · " (priority_label)
                    " · " (share_page.view_count) " view" @if share_page.view_count != 1 { "s" }
                }
            }

            // Parts table
            h2 { "Parts" }
            figure {
                table.responsive-table {
                    thead {
                        tr {
                            th { "Part" }
                            th { "Lvl" }
                            th { "SPD" }
                            th { "COR" }
                            th { "PWR" }
                            th { "QUA" }
                            th { "PIT" }
                            th { "Total" }
                        }
                    }
                    tbody {
                        @for p in parts {
                            tr {
                                td {
                                    small class="secondary" { (p.category) }
                                    " " span class=(part_rarity_class(&p.rarity)) { (p.part_name) }
                                }
                                td data-label="Lvl" { (p.level) }
                                td.stat-cell data-label="SPD" { (p.speed) }
                                td.stat-cell data-label="COR" { (p.cornering) }
                                td.stat-cell data-label="PWR" { (p.power_unit) }
                                td.stat-cell data-label="QUA" { (p.qualifying) }
                                td.stat-cell data-label="PIT" { (format!("{:.2}", p.pit_stop_time)) }
                                td.stat-cell data-label="Total" { strong { (p.total) } }
                            }
                        }
                    }
                    tfoot {
                        tr {
                            td { strong { "Total" } }
                            td {}
                            td.stat-cell data-label="SPD" { strong { (share_page.total_parts["speed"]) } }
                            td.stat-cell data-label="COR" { strong { (share_page.total_parts["cornering"]) } }
                            td.stat-cell data-label="PWR" { strong { (share_page.total_parts["power_unit"]) } }
                            td.stat-cell data-label="QUA" { strong { (share_page.total_parts["qualifying"]) } }
                            td.stat-cell data-label="PIT" { strong { (format!("{:.2}", share_page.total_parts["pit_stop_time"].as_f64().unwrap_or(0.0))) } }
                            td.stat-cell data-label="Total" { strong { (parts_total) } }
                        }
                    }
                }
            }

            // Drivers table
            @if !drivers.is_empty() {
                h2 { "Drivers" }
                figure {
                    table.responsive-table {
                        thead {
                            tr {
                                th { "Driver" }
                                th { "Lvl" }
                                th { "OVT" }
                                th { "DEF" }
                                th { "QUA" }
                                th { "RST" }
                                th { "TYR" }
                                th { "Total" }
                            }
                        }
                        tbody {
                            @for d in drivers {
                                @let d_rarity_class = DriverRarity::from_db(&d.rarity).map_or("", |r| r.css_class());
                                tr {
                                    td {
                                        small class="secondary" { (d.rarity) }
                                        " " span class=(d_rarity_class) { (d.driver_name) }
                                    }
                                    td data-label="Lvl" { (d.level) }
                                    td.stat-cell data-label="OVT" { (d.overtaking) }
                                    td.stat-cell data-label="DEF" { (d.defending) }
                                    td.stat-cell data-label="QUA" { (d.qualifying) }
                                    td.stat-cell data-label="RST" { (d.race_start) }
                                    td.stat-cell data-label="TYR" { (d.tyre_management) }
                                    td.stat-cell data-label="Total" { strong { (d.total) } }
                                }
                            }
                        }
                    }
                }
            }

            p {
                "Combined score: "
                strong { (parts_total + drivers_total) }
                " (" (parts_total) " parts + " (drivers_total) " drivers)"
            }

            // Compare with viewer's inventory
            @if !viewer_inventory.is_empty() {
                h2 { "Compare with Your Inventory" }
                figure {
                    table {
                        thead {
                            tr {
                                th { "Part" }
                                th { "Shared" }
                                th { "Yours" }
                                th { "Diff" }
                            }
                        }
                        tbody {
                            @for p in parts {
                                @let viewer_item = viewer_inventory.iter().find(|i| i.part_name == p.part_name);
                                tr {
                                    td { span class=(part_rarity_class(&p.rarity)) { (p.part_name) } }
                                    td { "L" (p.level) " (" (p.total) ")" }
                                    @if let Some(vi) = viewer_item {
                                        td {
                                            @if vi.level == p.level {
                                                "L" (vi.level)
                                            } @else {
                                                span class={ @if vi.level > p.level { "upgrade-positive" } @else { "compare-worst" } } {
                                                    "L" (vi.level)
                                                }
                                            }
                                        }
                                        td {
                                            @let diff = vi.level - p.level;
                                            @if diff > 0 {
                                                span class="upgrade-positive" { "+" (diff) " lvl" }
                                            } @else if diff < 0 {
                                                span class="compare-worst" { (diff) " lvl" }
                                            } @else {
                                                span class="secondary" { "=" }
                                            }
                                        }
                                    } @else {
                                        td class="compare-worst" { "Not owned" }
                                        td { "—" }
                                    }
                                }
                            }
                        }
                    }
                }
            } @else {
                p class="secondary" {
                    "Add parts to your inventory to see how your setup compares."
                }
            }

            div style="display:flex;gap:0.75rem;flex-wrap:wrap;margin-top:1.5rem" {
                a href="/optimizer" role="button" { "Try the Optimizer →" }
            }
        },
    )
}
