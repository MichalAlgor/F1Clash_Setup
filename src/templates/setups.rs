use maud::{html, Markup};

use crate::data;
use crate::drivers_data;
use crate::models::driver::DriverInventoryItem;
use crate::models::part::PartCategory;
use crate::models::setup::{InventoryItem, SetupWithStats};

pub fn list_page(setups: &[SetupWithStats]) -> Markup {
    super::layout::page(
        "Setups",
        html! {
            hgroup {
                h1 { "Car Setups" }
                p { "Create and compare configurations" }
            }

            a href="/setups/new" role="button" { "New Setup" }

            @if setups.is_empty() {
                p { "No setups yet. Add parts to your inventory, then create a setup." }
            } @else {
                figure {
                    table {
                        thead {
                            tr {
                                th { "Name" }
                                th { "SPD" }
                                th { "COR" }
                                th { "PWR" }
                                th { "QUA" }
                                th { "PIT (s)" }
                                th { "Total" }
                                th { "D.Total" }
                                th {}
                            }
                        }
                        tbody {
                            @for s in setups {
                                tr {
                                    td { a href={"/setups/" (s.setup.id)} { (s.setup.name) } }
                                    td { (s.stats.speed) }
                                    td { (s.stats.cornering) }
                                    td { (s.stats.power_unit) }
                                    td { (s.stats.qualifying) }
                                    td { (format!("{:.2}", s.stats.pit_stop_time)) }
                                    td { strong { (s.stats.total_performance()) } }
                                    td { (s.driver_stats.total()) }
                                    td {
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
        },
    )
}

pub fn form_page(
    inventory_by_category: &[(PartCategory, Vec<(InventoryItem, &data::LevelStats)>)],
    driver_items: &[DriverInventoryItem],
    setup: Option<&crate::models::setup::Setup>,
) -> Markup {
    let title = if setup.is_some() { "Edit Setup" } else { "New Setup" };
    let action = match setup {
        Some(s) => format!("/setups/{}", s.id),
        None => "/setups".to_string(),
    };

    super::layout::page(
        title,
        html! {
            h1 { (title) }
            form method="post" action=(action) {
                label for="name" { "Setup Name" }
                input type="text" id="name" name="name" required
                    value=[setup.map(|s| s.name.as_str())];

                h2 { "Parts" }
                @for (category, items) in inventory_by_category {
                    label for=(category.slug()) { (category.display_name()) }
                    select id=(category.slug()) name=(category.slug()) required {
                        option value="" { "Select a part…" }
                        @for (item, stats) in items {
                            option value=(item.id) {
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
                            @if let Some(driver_def) = drivers_data::find_driver_by_db(&item.driver_name, &item.rarity) {
                                @if let Some(stats) = driver_def.stats_for_level(item.level) {
                                    option value=(item.id) {
                                        (item.driver_name) " (" (driver_def.rarity.label()) ") Lvl " (item.level)
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
