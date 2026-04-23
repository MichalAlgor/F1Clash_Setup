use maud::{Markup, html};

use crate::auth::AuthStatus;
use crate::data::StatPriorities;
use crate::drivers_data::DriverRarity;
use crate::models::driver::{DriverInventoryItem, DriverStats};
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::InventoryItem;
use crate::optimizer_core::{DriverPriorities, OptimizeResult};

// ── Shared tab bar ────────────────────────────────────────────────────────────

fn tab_bar(active: &str) -> Markup {
    html! {
        div class="optimizer-tabs" {
            a href="/optimizer"
               class={"optimizer-tab" @if active == "presets" { " active" }} { "Presets" }
            a href="/optimizer/custom"
               class={"optimizer-tab" @if active == "custom" { " active" }} { "Custom" }
        }
    }
}

// ── Series limit form fragment ────────────────────────────────────────────────

fn series_limit_fields(action: &str) -> Markup {
    html! {
        form method="get" action=(action) {
            div style="max-width:220px" {
                label {
                    "Max part series (1–12)"
                    input type="number" name="max_part_series" min="1" max="12" value="12";
                }
            }
            button type="submit" { "Find Best Setups" }
        }
    }
}

// ── Form pages ────────────────────────────────────────────────────────────────

pub fn presets_form_page(auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Optimizer",
        auth,
        html! {
            hgroup {
                h1 { "Setup Optimizer" }
                p { "Find the best setups from your inventory" }
            }
            (tab_bar("presets"))
            p class="secondary" {
                "Runs 6 optimizations across Speed, Cornering and Power Unit — "
                "each paired with Qualifying. Drivers are optimized for highest total."
            }
            (series_limit_fields("/optimizer/presets"))
        },
    )
}

pub fn custom_form_page(auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Optimizer — Custom",
        auth,
        html! {
            hgroup {
                h1 { "Setup Optimizer" }
                p { "Find the best setup from your inventory" }
            }
            (tab_bar("custom"))

            form method="get" action="/optimizer/run" {
                fieldset {
                    legend { "Part stats" }
                    label {
                        input type="checkbox" name="speed" value="true";
                        " Speed"
                    }
                    label {
                        input type="checkbox" name="cornering" value="true";
                        " Cornering"
                    }
                    label {
                        input type="checkbox" name="power_unit" value="true";
                        " Power Unit"
                    }
                    label {
                        input type="checkbox" name="qualifying" value="true";
                        " Qualifying"
                    }
                }

                fieldset {
                    legend { "Driver stats" }
                    label {
                        input type="checkbox" name="overtaking" value="true";
                        " Overtaking"
                    }
                    label {
                        input type="checkbox" name="defending" value="true";
                        " Defending"
                    }
                    label {
                        input type="checkbox" name="d_qualifying" value="true";
                        " Qualifying"
                    }
                    label {
                        input type="checkbox" name="race_start" value="true";
                        " Race Start"
                    }
                    label {
                        input type="checkbox" name="tyre_management" value="true";
                        " Tyre Mgmt"
                    }
                }

                fieldset {
                    legend { "Series limits" }
                    div class="series-limits-grid" {
                        label {
                            "Max part series (1–12)"
                            input type="number" name="max_part_series" min="1" max="12" value="12";
                        }
                        label {
                            "Max driver series (1–12)"
                            input type="number" name="max_driver_series" min="1" max="12" value="12";
                        }
                    }
                }

                button type="submit" { "Find Best Setup" }
            }
        },
    )
}

// ── Presets result ────────────────────────────────────────────────────────────

pub struct PresetResult {
    pub label: &'static str,
    pub result: Option<OptimizeResult>,
}

pub fn presets_result_page(presets: &[PresetResult], auth: &AuthStatus) -> Markup {
    let groups = [
        ("Speed", 0usize, 1usize),
        ("Cornering", 2, 3),
        ("Power Unit", 4, 5),
    ];

    super::layout::page(
        "Optimizer — Presets",
        auth,
        html! {
            hgroup {
                h1 { "Optimized Setups" }
                p { "6 presets across Speed, Cornering and Power Unit" }
            }
            (tab_bar("presets"))

            a href="/optimizer" role="button" class="outline back-link" {
                "← Change series limits"
            }

            @for (group_name, a, b) in &groups {
                div class="preset-group" {
                    h2 { (group_name) }
                    div class="preset-pair" {
                        (preset_card(&presets[*a]))
                        (preset_card(&presets[*b]))
                    }
                }
            }
        },
    )
}

fn preset_card(preset: &PresetResult) -> Markup {
    html! {
        div class="preset-card" {
            h3 style="margin-top:0;font-size:0.95rem" { (preset.label) }
            @match &preset.result {
                None => {
                    p class="secondary" {
                        "No parts in inventory for one or more categories."
                    }
                }
                Some(r) => {
                    // Parts table
                    figure class="preset-figure" {
                        table class="responsive-table preset-parts-table" {
                            thead {
                                tr {
                                    th { "Part" }
                                    th { "Lvl" }
                                    th class="stat-header" { "SPD" }
                                    th class="stat-header" { "COR" }
                                    th class="stat-header" { "PWR" }
                                    th class="stat-header" { "QUA" }
                                    th class="stat-header" { "PIT" }
                                    th { "Tot" }
                                }
                            }
                            tbody {
                                @for (cat, item, stats, rarity_class) in &r.part_picks {
                                    tr {
                                        td { small class="secondary" { (cat.display_name()) } " " span class=(*rarity_class) { (item.part_name.clone()) } }
                                        td data-label="Lvl" { (item.level) }
                                        td data-label="SPD" class="stat-cell" { (stats.speed) }
                                        td data-label="COR" class="stat-cell" { (stats.cornering) }
                                        td data-label="PWR" class="stat-cell" { (stats.power_unit) }
                                        td data-label="QUA" class="stat-cell" { (stats.qualifying) }
                                        td data-label="PIT" class="stat-cell" { (format!("{:.2}", stats.pit_stop_time)) }
                                        td data-label="Total" { strong { (stats.total_performance()) } }
                                    }
                                }
                            }
                            tfoot {
                                tr {
                                    td colspan="2" data-label="Parts" { strong { "Total" } }
                                    td data-label="SPD" class="stat-cell" { strong { (r.total_parts.speed) } }
                                    td data-label="COR" class="stat-cell" { strong { (r.total_parts.cornering) } }
                                    td data-label="PWR" class="stat-cell" { strong { (r.total_parts.power_unit) } }
                                    td data-label="QUA" class="stat-cell" { strong { (r.total_parts.qualifying) } }
                                    td data-label="PIT" class="stat-cell" { strong { (format!("{:.2}", r.total_parts.pit_stop_time)) } }
                                    td data-label="Total" { strong { (r.total_parts.total_performance()) } }
                                }
                            }
                        }
                    }

                    // Parts score
                    p class="preset-score" {
                        "Total: "
                        strong { (r.total_parts.total_performance()) }
                        small class="secondary" {
                            "  PIT " (format!("{:.2}s", r.total_parts.pit_stop_time))
                        }
                    }

                    // Save / Share form (parts only)
                    form method="post" action="/optimizer/save" class="preset-save-form" {
                        input #{"save-toggle-" (preset.label.replace(' ', "-"))} type="checkbox" class="save-toggle" {}
                        div class="save-name-row" {
                            span class="save-name-display" { "Optimized (" (preset.label) ")" }
                            label class="save-edit-btn" for={"save-toggle-" (preset.label.replace(' ', "-"))} { "✏" }
                        }
                        input type="text" name="name" required class="save-name-edit"
                            value={"Optimized (" (preset.label) ")"};
                        div class="preset-form-btns" {
                            button type="submit" class="save-form-btn outline" { "Save" }
                            button type="submit" formaction="/optimizer/share" class="save-form-btn outline" { "Share" }
                        }
                        @for (cat, item, _, _) in &r.part_picks {
                            @if item.id != 0 {
                                input type="hidden" name=(format!("{}_id", cat.slug())) value=(item.id);
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Custom result page (unchanged) ────────────────────────────────────────────

pub fn result_page(
    part_priorities: &StatPriorities,
    driver_priorities: &DriverPriorities,
    part_picks: &[(PartCategory, InventoryItem, Stats, &'static str)],
    driver1: Option<&(DriverInventoryItem, DriverStats)>,
    driver2: Option<&(DriverInventoryItem, DriverStats)>,
    total_parts: &Stats,
    total_drivers: &DriverStats,
    auth: &AuthStatus,
) -> Markup {
    let part_labels = part_priorities.labels().join(", ");
    let driver_labels = driver_priorities.labels().join(", ");
    let all_labels = {
        let mut v = Vec::new();
        if !part_labels.is_empty() {
            v.push(part_labels.clone());
        }
        if !driver_labels.is_empty() {
            v.push(driver_labels.clone());
        }
        if v.is_empty() {
            "Total".to_string()
        } else {
            v.join(", ")
        }
    };

    super::layout::page(
        "Optimizer Result",
        auth,
        html! {
            hgroup {
                h1 { "Optimized Setup" }
                p { "Custom optimizer result" }
            }
            (tab_bar("custom"))

            @if part_picks.is_empty() {
                p { "No parts in inventory for one or more categories. Add parts first!" }
            } @else {
                div class="preset-card" {
                    h3 style="margin-top:0;font-size:0.95rem" { (all_labels) }

                    // Parts table
                    figure class="preset-figure" {
                        table class="responsive-table preset-parts-table" {
                            thead {
                                tr {
                                    th { "Part" }
                                    th { "Lvl" }
                                    th class="stat-header" { "SPD" }
                                    th class="stat-header" { "COR" }
                                    th class="stat-header" { "PWR" }
                                    th class="stat-header" { "QUA" }
                                    th class="stat-header" { "PIT" }
                                    th { "Tot" }
                                }
                            }
                            tbody {
                                @for (cat, item, stats, rarity_class) in part_picks {
                                    tr {
                                        td { small class="secondary" { (cat.display_name()) } " " span class=(*rarity_class) { (item.part_name.clone()) } }
                                        td data-label="Lvl" { (item.level) }
                                        td data-label="SPD" class="stat-cell" { (stats.speed) }
                                        td data-label="COR" class="stat-cell" { (stats.cornering) }
                                        td data-label="PWR" class="stat-cell" { (stats.power_unit) }
                                        td data-label="QUA" class="stat-cell" { (stats.qualifying) }
                                        td data-label="PIT" class="stat-cell" { (format!("{:.2}", stats.pit_stop_time)) }
                                        td data-label="Total" { strong { (stats.total_performance()) } }
                                    }
                                }
                            }
                            tfoot {
                                tr {
                                    td colspan="2" data-label="Parts" { strong { "Total" } }
                                    td data-label="SPD" class="stat-cell" { strong { (total_parts.speed) } }
                                    td data-label="COR" class="stat-cell" { strong { (total_parts.cornering) } }
                                    td data-label="PWR" class="stat-cell" { strong { (total_parts.power_unit) } }
                                    td data-label="QUA" class="stat-cell" { strong { (total_parts.qualifying) } }
                                    td data-label="PIT" class="stat-cell" { strong { (format!("{:.2}", total_parts.pit_stop_time)) } }
                                    td data-label="Total" { strong { (total_parts.total_performance()) } }
                                }
                            }
                        }
                    }

                    // Drivers table
                    @if driver1.is_some() || driver2.is_some() {
                        figure class="preset-figure" {
                            table class="responsive-table preset-parts-table" {
                                thead {
                                    tr {
                                        th { "Driver" }
                                        th { "Lvl" }
                                        th class="stat-header" { "OVT" }
                                        th class="stat-header" { "DEF" }
                                        th class="stat-header" { "QUA" }
                                        th class="stat-header" { "RST" }
                                        th class="stat-header" { "TYR" }
                                        th { "Tot" }
                                    }
                                }
                                tbody {
                                    @for driver_opt in &[driver1, driver2] {
                                        @if let Some((item, stats)) = driver_opt {
                                            @let d_rarity = DriverRarity::from_db(&item.rarity);
                                            tr {
                                                td { small class="secondary" { (item.rarity.clone()) } " " span class=[d_rarity.map(|r| r.css_class())] { (item.driver_name.clone()) } }
                                                td data-label="Lvl" { (item.level) }
                                                td data-label="OVT" class="stat-cell" { (stats.overtaking) }
                                                td data-label="DEF" class="stat-cell" { (stats.defending) }
                                                td data-label="QUA" class="stat-cell" { (stats.qualifying) }
                                                td data-label="RST" class="stat-cell" { (stats.race_start) }
                                                td data-label="TYR" class="stat-cell" { (stats.tyre_management) }
                                                td data-label="Total" { strong { (stats.total()) } }
                                            }
                                        }
                                    }
                                }
                                tfoot {
                                    tr {
                                        td colspan="2" data-label="Drivers" { strong { "Total" } }
                                        td data-label="OVT" class="stat-cell" { strong { (total_drivers.overtaking) } }
                                        td data-label="DEF" class="stat-cell" { strong { (total_drivers.defending) } }
                                        td data-label="QUA" class="stat-cell" { strong { (total_drivers.qualifying) } }
                                        td data-label="RST" class="stat-cell" { strong { (total_drivers.race_start) } }
                                        td data-label="TYR" class="stat-cell" { strong { (total_drivers.tyre_management) } }
                                        td data-label="Total" { strong { (total_drivers.total()) } }
                                    }
                                }
                            }
                        }
                    }

                    // Score summary
                    p class="preset-score" {
                        "Total: "
                        strong { (total_parts.total_performance() + total_drivers.total()) }
                        small class="secondary" {
                            "  Parts " (total_parts.total_performance())
                            @if total_drivers.total() > 0 {
                                " · Drivers " (total_drivers.total())
                            }
                            "  PIT " (format!("{:.2}s", total_parts.pit_stop_time))
                        }
                    }

                    // Save / Share form
                    form method="post" action="/optimizer/save" class="preset-save-form" {
                        input id="custom-save-toggle" type="checkbox" class="save-toggle" {}
                        div class="save-name-row" {
                            span class="save-name-display" { "Optimized (" (all_labels) ")" }
                            label class="save-edit-btn" for="custom-save-toggle" { "✏" }
                        }
                        input type="text" name="name" required class="save-name-edit"
                            value=(format!("Optimized ({all_labels})"));
                        div class="preset-form-btns" {
                            button type="submit" class="save-form-btn outline" { "Save" }
                            button type="submit" formaction="/optimizer/share" class="save-form-btn outline" { "Share" }
                        }
                        @for (cat, item, _, _) in part_picks {
                            @if item.id != 0 {
                                input type="hidden" name=(format!("{}_id", cat.slug())) value=(item.id);
                            }
                        }
                        @if let Some((item, _)) = driver1 {
                            input type="hidden" name="driver1_id" value=(item.id);
                        }
                        @if let Some((item, _)) = driver2 {
                            input type="hidden" name="driver2_id" value=(item.id);
                        }
                        @if part_priorities.speed { input type="hidden" name="speed" value="true"; }
                        @if part_priorities.cornering { input type="hidden" name="cornering" value="true"; }
                        @if part_priorities.power_unit { input type="hidden" name="power_unit" value="true"; }
                        @if part_priorities.qualifying { input type="hidden" name="qualifying" value="true"; }
                    }
                }

                a href="/optimizer/custom" role="button" class="outline" style="margin-top:1rem;display:inline-block" {
                    "← Try different priorities"
                }
            }
        },
    )
}
