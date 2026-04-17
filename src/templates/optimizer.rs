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
                    div style="display:grid;grid-template-columns:1fr 1fr;gap:1rem;" {
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

            a href="/optimizer" role="button" class="outline" style="margin-bottom:1rem;display:inline-block" {
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
                    figure style="margin:0 0 0.5rem" {
                        table {
                            thead {
                                tr {
                                    th { "Part" }
                                    th { "Lvl" }
                                    th { "SPD" }
                                    th { "COR" }
                                    th { "PWR" }
                                    th { "QUA" }
                                    th { "PIT" }
                                    th { "Tot" }
                                }
                            }
                            tbody {
                                @for (cat, item, stats, rarity_class) in &r.part_picks {
                                    tr {
                                        td { small class="secondary" { (cat.display_name()) } " " span class=(*rarity_class) { (item.part_name.clone()) } }
                                        td { (item.level) }
                                        td { (stats.speed) }
                                        td { (stats.cornering) }
                                        td { (stats.power_unit) }
                                        td { (stats.qualifying) }
                                        td { (format!("{:.2}", stats.pit_stop_time)) }
                                        td { strong { (stats.total_performance()) } }
                                    }
                                }
                            }
                            tfoot {
                                tr {
                                    td colspan="2" { strong { "Total" } }
                                    td { strong { (r.total_parts.speed) } }
                                    td { strong { (r.total_parts.cornering) } }
                                    td { strong { (r.total_parts.power_unit) } }
                                    td { strong { (r.total_parts.qualifying) } }
                                    td { strong { (format!("{:.2}", r.total_parts.pit_stop_time)) } }
                                    td { strong { (r.total_parts.total_performance()) } }
                                }
                            }
                        }
                    }

                    // Parts score
                    p style="margin:0.25rem 0" {
                        "Total: "
                        strong { (r.total_parts.total_performance()) }
                        small class="secondary" {
                            "  PIT " (format!("{:.2}s", r.total_parts.pit_stop_time))
                        }
                    }

                    // Save form (parts only)
                    form method="post" action="/optimizer/save" style="margin-top:0.5rem" {
                        div style="display:flex;gap:0.5rem;align-items:flex-end" {
                            div style="flex:1" {
                                input type="text" name="name" required
                                    value={"Optimized (" (preset.label) ")"};
                            }
                            button type="submit" style="white-space:nowrap" { "Save" }
                        }
                        @for (cat, item, _, _) in &r.part_picks {
                            input type="hidden" name=(format!("{}_id", cat.slug())) value=(item.id);
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

            @if part_priorities.any_selected() || driver_priorities.any_selected() {
                p {
                    @if !part_labels.is_empty() {
                        "Part priorities: " strong { (part_labels) }
                    }
                    @if !part_labels.is_empty() && !driver_labels.is_empty() {
                        " | "
                    }
                    @if !driver_labels.is_empty() {
                        "Driver priorities: " strong { (driver_labels) }
                    }
                }
            } @else {
                p { "No priorities selected — optimizing for highest totals" }
            }

            @if part_picks.is_empty() {
                p { "No parts in inventory for one or more categories. Add parts first!" }
            } @else {
                h2 { "Parts" }
                figure {
                    table {
                        thead {
                            tr {
                                th { "Category" }
                                th { "Part" }
                                th { "Lvl" }
                                th { "SPD" }
                                th { "COR" }
                                th { "PWR" }
                                th { "QUA" }
                                th { "PIT (s)" }
                                th { "Total" }
                            }
                        }
                        tbody {
                            @for (cat, item, stats, rarity_class) in part_picks {
                                tr {
                                    td { (cat.display_name()) }
                                    td { strong class=(*rarity_class) { (item.part_name.clone()) } }
                                    td { (item.level) }
                                    td { (stats.speed) }
                                    td { (stats.cornering) }
                                    td { (stats.power_unit) }
                                    td { (stats.qualifying) }
                                    td { (format!("{:.2}", stats.pit_stop_time)) }
                                    td { (stats.total_performance()) }
                                }
                            }
                        }
                        tfoot {
                            tr {
                                td colspan="3" { strong { "Total" } }
                                td { strong { (total_parts.speed) } }
                                td { strong { (total_parts.cornering) } }
                                td { strong { (total_parts.power_unit) } }
                                td { strong { (total_parts.qualifying) } }
                                td { strong { (format!("{:.2}", total_parts.pit_stop_time)) } }
                                td { strong { (total_parts.total_performance()) } }
                            }
                        }
                    }
                }

                h2 { "Drivers" }
                @if driver1.is_none() && driver2.is_none() {
                    p { "No drivers in inventory." }
                } @else {
                    figure {
                        table {
                            thead {
                                tr {
                                    th { "Driver" }
                                    th { "Rarity" }
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
                                @for driver_opt in &[driver1, driver2] {
                                    @if let Some((item, stats)) = driver_opt {
                                        @let d_rarity = DriverRarity::from_db(&item.rarity);
                                        tr {
                                            td class=[d_rarity.map(|r| r.css_class())] { strong { (item.driver_name.clone()) } }
                                            td { (item.rarity) }
                                            td { (item.level) }
                                            td { (stats.overtaking) }
                                            td { (stats.defending) }
                                            td { (stats.qualifying) }
                                            td { (stats.race_start) }
                                            td { (stats.tyre_management) }
                                            td { (stats.total()) }
                                        }
                                    }
                                }
                            }
                            tfoot {
                                tr {
                                    td colspan="3" { strong { "Total" } }
                                    td { strong { (total_drivers.overtaking) } }
                                    td { strong { (total_drivers.defending) } }
                                    td { strong { (total_drivers.qualifying) } }
                                    td { strong { (total_drivers.race_start) } }
                                    td { strong { (total_drivers.tyre_management) } }
                                    td { strong { (total_drivers.total()) } }
                                }
                            }
                        }
                    }
                }

                p {
                    "Combined score: "
                    strong { (total_parts.total_performance() + total_drivers.total()) }
                    " (" (total_parts.total_performance()) " parts + " (total_drivers.total()) " drivers)"
                }

                h2 { "Save this setup" }
                form method="post" action="/optimizer/save" {
                    label for="name" { "Setup Name" }
                    input type="text" id="name" name="name" required
                        value=(format!("Optimized ({all_labels})"));

                    @for (cat, item, _, _) in part_picks {
                        input type="hidden" name=(format!("{}_id", cat.slug())) value=(item.id);
                    }
                    @if let Some((item, _)) = driver1 {
                        input type="hidden" name="driver1_id" value=(item.id);
                    }
                    @if let Some((item, _)) = driver2 {
                        input type="hidden" name="driver2_id" value=(item.id);
                    }

                    button type="submit" { "Save Setup" }
                }

                a href="/optimizer/custom" role="button" class="outline" { "← Try different priorities" }
            }
        },
    )
}
