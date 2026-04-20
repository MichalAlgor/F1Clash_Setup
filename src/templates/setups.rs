use maud::{Markup, html};

use crate::auth::AuthStatus;
use crate::drivers_data::DriverRarity;
use crate::models::driver::{DriverInventoryItem, OwnedDriverDefinition};
use crate::models::part::{OwnedLevelStats, PartCategory};
use crate::models::setup::{InventoryItem, SetupWithStats};

pub fn list_page(setups: &[SetupWithStats], auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Setups",
        auth,
        html! {
            hgroup {
                h1 { "Car Setups" }
                p { "Create and compare configurations" }
            }

            div class="setups-actions" {
                a href="/setups/new" role="button" { "New Setup" }
                @if setups.len() >= 2 {
                    a #compare-link role="button" class="outline" href="/setups/compare" style="display:none" { "Compare" }
                }
            }

            @if setups.is_empty() {
                p { "No setups yet. Add parts to your inventory, then create a setup." }
            } @else {
                figure {
                    table.responsive-table #setups-table {
                        thead {
                            tr {
                                @if setups.len() >= 2 {
                                    th class="compare-col" { "" }
                                }
                                th { "Name" }
                                th { "SPD" }
                                th { "COR" }
                                th { "PWR" }
                                th { "QUA" }
                                th { "PIT (s)" }
                                th { "P.Total" }
                                th { "D.Total" }
                                th { "Score" }
                                th {}
                            }
                        }
                        tbody {
                            @for s in setups {
                                tr {
                                    @if setups.len() >= 2 {
                                        td class="compare-col action-cell" {
                                            input type="checkbox" class="compare-check"
                                                value=(s.setup.id);
                                        }
                                    }
                                    td { a href={"/setups/" (s.setup.id)} { (s.setup.name) } }
                                    td.stat-cell data-label="SPD" { (s.stats.speed) }
                                    td.stat-cell data-label="COR" { (s.stats.cornering) }
                                    td.stat-cell data-label="PWR" { (s.stats.power_unit) }
                                    td.stat-cell data-label="QUA" { (s.stats.qualifying) }
                                    td.stat-cell data-label="PIT" { (format!("{:.2}", s.stats.pit_stop_time)) }
                                    td.stat-cell data-label="P.Tot" { (s.stats.total_performance()) }
                                    td.stat-cell data-label="D.Tot" { (s.driver_stats.total()) }
                                    td.stat-cell data-label="Score" { strong { (s.stats.total_performance() + s.driver_stats.total()) } }
                                    td.action-cell {
                                        button.outline.secondary
                                            hx-delete={"/setups/" (s.setup.id)}
                                            hx-confirm="Delete this setup?"
                                            hx-target="closest tr"
                                            hx-swap="outerHTML"
                                        { "×" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Compare script — minimal vanilla JS for checkbox aggregation
            @if setups.len() >= 2 {
                script {
                    (maud::PreEscaped(r#"
(function() {
    var link = document.getElementById('compare-link');
    document.querySelectorAll('.compare-check').forEach(function(cb) {
        cb.addEventListener('change', function() {
            var checked = document.querySelectorAll('.compare-check:checked');
            if (checked.length >= 2) {
                var ids = Array.from(checked).map(function(c) { return c.value; }).join(',');
                link.href = '/setups/compare?ids=' + ids;
                link.textContent = 'Compare (' + checked.length + ')';
                link.style.display = '';
            } else {
                link.style.display = 'none';
            }
        });
    });
})();
                    "#))
                }
            }
        },
    )
}

pub fn form_page(
    inventory_by_category: &[(PartCategory, Vec<(InventoryItem, OwnedLevelStats)>)],
    driver_items: &[DriverInventoryItem],
    drivers_catalog: &[OwnedDriverDefinition],
    setup: Option<&crate::models::setup::Setup>,
    auth: &AuthStatus,
) -> Markup {
    let title = if setup.is_some() {
        "Edit Setup"
    } else {
        "New Setup"
    };
    let action = match setup {
        Some(s) => format!("/setups/{}", s.id),
        None => "/setups".to_string(),
    };

    super::layout::page(
        title,
        auth,
        html! {
            h1 { (title) }
            form method="post" action=(action) {
                label for="name" { "Setup Name" }
                input type="text" id="name" name="name" required
                    value=[setup.map(|s| s.name.as_str())];

                h2 { "Parts" }
                @for (category, items) in inventory_by_category {
                    @let current_id = setup.and_then(|s| {
                        match category {
                            crate::models::part::PartCategory::Engine     => s.engine_id,
                            crate::models::part::PartCategory::FrontWing  => s.front_wing_id,
                            crate::models::part::PartCategory::RearWing   => s.rear_wing_id,
                            crate::models::part::PartCategory::Suspension => s.suspension_id,
                            crate::models::part::PartCategory::Brakes     => s.brakes_id,
                            crate::models::part::PartCategory::Gearbox    => s.gearbox_id,
                            crate::models::part::PartCategory::Battery    => s.battery_id,
                        }
                    });
                    label for=(category.slug()) { (category.display_name()) }
                    select id=(category.slug()) name=(category.slug()) {
                        option value="" selected[current_id.is_none()] { "Default (1/1/1/1 · 1.00s pit)" }
                        @for (item, stats) in items {
                            option value=(item.id) selected[current_id == Some(item.id)] {
                                (item.part_name) " Lvl " (item.level)
                                " — " (stats.speed + stats.cornering + stats.power_unit + stats.qualifying) " perf"
                                " / " (format!("{:.2}", stats.pit_stop_time)) "s pit"
                            }
                        }
                    }
                }

                h2 { "Drivers" }
                @for slot in &["driver1_id", "driver2_id"] {
                    @let label_text = if *slot == "driver1_id" { "Driver 1" } else { "Driver 2" };
                    label for=(*slot) { (label_text) }
                    select id=(*slot) name=(*slot) {
                        option value="" { "No driver" }
                        @for item in driver_items {
                            @if let Some(driver_def) = drivers_catalog.iter().find(|d| d.name == item.driver_name && d.rarity == item.rarity) {
                                @if let Some(stats) = driver_def.stats_for_level(item.level) {
                                    @let rarity_label = DriverRarity::from_db(&driver_def.rarity).map_or(driver_def.rarity.as_str(), |r| r.label());
                                    option value=(item.id) {
                                        (item.driver_name) " (" (rarity_label) ") Lvl " (item.level)
                                        " — " (stats.total()) " total"
                                    }
                                }
                            }
                        }
                    }
                }

                button type="submit" { "Save Setup" }
            }
        },
    )
}

pub fn comparison_page(setups: &[SetupWithStats], auth: &AuthStatus) -> Markup {
    if setups.is_empty() {
        return super::layout::page(
            "Compare Setups",
            auth,
            html! {
                h1 { "Compare Setups" }
                p { "No setups found." }
                a href="/setups" role="button" class="outline" { "← Back to setups" }
            },
        );
    }

    // Collect all stat values per row for highlighting
    let speeds: Vec<i32> = setups.iter().map(|s| s.stats.speed).collect();
    let corners: Vec<i32> = setups.iter().map(|s| s.stats.cornering).collect();
    let pwrs: Vec<i32> = setups.iter().map(|s| s.stats.power_unit).collect();
    let quals: Vec<i32> = setups.iter().map(|s| s.stats.qualifying).collect();
    let pits: Vec<f64> = setups.iter().map(|s| s.stats.pit_stop_time).collect();
    let ptotals: Vec<i32> = setups.iter().map(|s| s.stats.total_performance()).collect();
    let dtotals: Vec<i32> = setups.iter().map(|s| s.driver_stats.total()).collect();
    let scores: Vec<i32> = setups
        .iter()
        .map(|s| s.stats.total_performance() + s.driver_stats.total())
        .collect();

    super::layout::page(
        "Compare Setups",
        auth,
        html! {
            hgroup {
                h1 { "Compare Setups" }
                p { "Side-by-side stat comparison" }
            }
            a href="/setups" role="button" class="outline back-link" { "← Back to setups" }

            div style="overflow-x:auto;-webkit-overflow-scrolling:touch" {
                table class="compare-table" {
                    thead {
                        tr {
                            th { "Stat" }
                            @for s in setups {
                                th {
                                    a href={"/setups/" (s.setup.id)} { (s.setup.name) }
                                }
                            }
                        }
                    }
                    tbody {
                        (compare_row("SPD", &speeds.iter().map(|v| v.to_string()).collect::<Vec<_>>(), false))
                        (compare_row("COR", &corners.iter().map(|v| v.to_string()).collect::<Vec<_>>(), false))
                        (compare_row("PWR", &pwrs.iter().map(|v| v.to_string()).collect::<Vec<_>>(), false))
                        (compare_row("QUA", &quals.iter().map(|v| v.to_string()).collect::<Vec<_>>(), false))
                        (compare_row_f("PIT (s)", &pits, true))
                        (compare_row("P.Total", &ptotals.iter().map(|v| v.to_string()).collect::<Vec<_>>(), false))
                        (compare_row("D.Total", &dtotals.iter().map(|v| v.to_string()).collect::<Vec<_>>(), false))
                        (compare_row("Score", &scores.iter().map(|v| v.to_string()).collect::<Vec<_>>(), false))
                    }
                }
            }
        },
    )
}

/// Render a comparison table row with best/worst highlighting (int values).
/// `lower_is_better`: true for PIT stop time.
fn compare_row(label: &str, values: &[String], lower_is_better: bool) -> Markup {
    let parsed: Vec<i64> = values
        .iter()
        .filter_map(|v| v.parse::<i64>().ok())
        .collect();
    let best = if lower_is_better {
        parsed.iter().copied().min()
    } else {
        parsed.iter().copied().max()
    };
    let worst = if lower_is_better {
        parsed.iter().copied().max()
    } else {
        parsed.iter().copied().min()
    };
    html! {
        tr {
            td { strong { (label) } }
            @for (val, parsed_val) in values.iter().zip(parsed.iter()) {
                @let is_best = Some(*parsed_val) == best && parsed.iter().filter(|&&v| v == *parsed_val).count() < parsed.len();
                @let is_worst = Some(*parsed_val) == worst && parsed.iter().filter(|&&v| v == *parsed_val).count() < parsed.len();
                td class={
                    @if is_best { "compare-best" }
                    @else if is_worst { "compare-worst" }
                    @else { "" }
                } { (val) }
            }
        }
    }
}

/// Render a comparison row for float values (PIT stop time).
fn compare_row_f(label: &str, values: &[f64], lower_is_better: bool) -> Markup {
    let display: Vec<String> = values.iter().map(|v| format!("{:.2}", v)).collect();
    let best = if lower_is_better {
        values.iter().copied().reduce(f64::min)
    } else {
        values.iter().copied().reduce(f64::max)
    };
    let worst = if lower_is_better {
        values.iter().copied().reduce(f64::max)
    } else {
        values.iter().copied().reduce(f64::min)
    };
    html! {
        tr {
            td { strong { (label) } }
            @for (val, raw) in display.iter().zip(values.iter()) {
                @let is_best = best.map_or(false, |b| (raw - b).abs() < 0.001) && values.iter().filter(|&&v| (v - raw).abs() < 0.001).count() < values.len();
                @let is_worst = worst.map_or(false, |w| (raw - w).abs() < 0.001) && values.iter().filter(|&&v| (v - raw).abs() < 0.001).count() < values.len();
                td class={
                    @if is_best { "compare-best" }
                    @else if is_worst { "compare-worst" }
                    @else { "" }
                } { (val) }
            }
        }
    }
}
