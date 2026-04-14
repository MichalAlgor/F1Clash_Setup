use maud::{html, Markup};

use crate::data::StatPriorities;
use crate::drivers_data;
use crate::models::driver::{DriverInventoryItem, DriverStats};
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::InventoryItem;
use crate::routes::optimizer::DriverPriorities;

pub fn form_page() -> Markup {
    super::layout::page(
        "Optimizer",
        html! {
            hgroup {
                h1 { "Setup Optimizer" }
                p { "Select stats to prioritize, then find the best setup from your inventory" }
            }

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
                            input type="number" name="max_part_series" min="1" max="12" placeholder="any";
                        }
                        label {
                            "Max driver series (1–12)"
                            input type="number" name="max_driver_series" min="1" max="12" placeholder="any";
                        }
                    }
                }

                button type="submit" { "Find Best Setup" }
            }
        },
    )
}

pub fn result_page(
    part_priorities: &StatPriorities,
    driver_priorities: &DriverPriorities,
    part_picks: &[(PartCategory, InventoryItem, Stats)],
    driver1: Option<&(DriverInventoryItem, DriverStats)>,
    driver2: Option<&(DriverInventoryItem, DriverStats)>,
    total_parts: &Stats,
    total_drivers: &DriverStats,
) -> Markup {
    let part_labels = part_priorities.labels().join(", ");
    let driver_labels = driver_priorities.labels().join(", ");
    let all_labels = {
        let mut v = Vec::new();
        if !part_labels.is_empty() { v.push(part_labels.clone()); }
        if !driver_labels.is_empty() { v.push(driver_labels.clone()); }
        if v.is_empty() { "Total".to_string() } else { v.join(", ") }
    };

    super::layout::page(
        "Optimizer Result",
        html! {
            h1 { "Optimized Setup" }

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
                // Parts table
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
                            @for (cat, item, stats) in part_picks {
                                tr {
                                    td { (cat.display_name()) }
                                    td { strong { (item.part_name.clone()) } }
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

                // Drivers table
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
                                        @if let Some(def) = drivers_data::find_driver_by_db(&item.driver_name, &item.rarity) {
                                            tr {
                                                td class=(def.rarity.css_class()) { strong { (item.driver_name.clone()) } }
                                                td { (def.rarity.label()) }
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

                // Combined score
                p {
                    "Combined score: "
                    strong { (total_parts.total_performance() + total_drivers.total()) }
                    " (" (total_parts.total_performance()) " parts + " (total_drivers.total()) " drivers)"
                }

                // Save form
                h2 { "Save this setup" }
                form method="post" action="/optimizer/save" {
                    label for="name" { "Setup Name" }
                    input type="text" id="name" name="name" required
                        value=(format!("Optimized ({all_labels})"));

                    @for (cat, item, _) in part_picks {
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

                a href="/optimizer" role="button" class="outline" { "← Try different priorities" }
            }
        },
    )
}
