use maud::{html, Markup};

use crate::drivers_data::{self, DriverCategory, DriverRarity};
use crate::models::driver::DriverInventoryItem;

pub fn list_page(items: &[DriverInventoryItem]) -> Markup {
    super::layout::page(
        "Drivers",
        html! {
            hgroup {
                h1 { "Driver Inventory" }
                p { "Drivers you own and their current levels" }
            }

            a href="/drivers/bulk" role="button" { "Manage All Drivers" }

            div class="category-grid" {
                @for category in DriverCategory::all() {
                    @let cat_items: Vec<_> = {
                        let mut v: Vec<_> = items.iter()
                            .filter(|item| {
                                drivers_data::find_driver_by_db(&item.driver_name, &item.rarity)
                                    .is_some_and(|d| d.rarity.category() == *category)
                            })
                            .collect();
                        v.sort_by_key(|item| {
                            let r = DriverRarity::from_db(&item.rarity).unwrap_or(DriverRarity::Common);
                            drivers_data::driver_catalog_index(&item.driver_name, &r)
                        });
                        v
                    };

                    @if !cat_items.is_empty() {
                        section {
                            h2 { (category.display_name()) }
                            figure {
                                table {
                                    thead {
                                        tr {
                                            th { "Name" }
                                            th { "Rarity" }
                                            th { "Series" }
                                            th { "Lvl" }
                                            th { "OVT" }
                                            th { "DEF" }
                                            th { "QUA" }
                                            th { "RST" }
                                            th { "TYR" }
                                            th { "Total" }
                                            th {}
                                        }
                                    }
                                    tbody {
                                        @for item in &cat_items {
                                            @if let Some(driver_def) = drivers_data::find_driver_by_db(&item.driver_name, &item.rarity) {
                                                @if let Some(stats) = driver_def.stats_for_level(item.level) {
                                                    tr {
                                                        td class=(driver_def.rarity.css_class()) { (item.driver_name) }
                                                        td { (driver_def.rarity.label()) }
                                                        td { (driver_def.series) }
                                                        td {
                                                            form method="post" action={"/drivers/" (item.id) "/level"} style="display:inline;margin:0" {
                                                                select name="level" onchange="this.form.submit()" class="inline-select" {
                                                                    @for l in driver_def.levels {
                                                                        option value=(l.level) selected[l.level == item.level] {
                                                                            (l.level)
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        td { (stats.overtaking) }
                                                        td { (stats.defending) }
                                                        td { (stats.qualifying) }
                                                        td { (stats.race_start) }
                                                        td { (stats.tyre_management) }
                                                        td { strong { (stats.total()) } }
                                                        td {
                                                            button.outline.secondary
                                                                hx-delete={"/drivers/" (item.id)}
                                                                hx-confirm="Remove this driver?"
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
                            }
                        }
                    }
                }
            }
        },
    )
}

pub fn bulk_page(current_inventory: &[DriverInventoryItem]) -> Markup {
    super::layout::page(
        "Manage All Drivers",
        html! {
            h1 { "Manage All Drivers" }
            p { "Set the level for each driver you own. Leave at \"—\" for drivers you don't have." }

            form method="post" action="/drivers/bulk" {
                div class="category-grid" {
                    @for category in DriverCategory::all() {
                        @let drivers = drivers_data::drivers_by_category(*category);
                        @if !drivers.is_empty() {
                            section {
                                h2 { (category.display_name()) }
                                figure {
                                    table class="bulk-table" {
                                        thead {
                                            tr {
                                                th { "Name" }
                                                th { "Rarity" }
                                                th { "Series" }
                                                th { "Level" }
                                            }
                                        }
                                        tbody {
                                            @for driver_def in &drivers {
                                                @let current_level = current_inventory.iter()
                                                    .find(|i| i.driver_name == driver_def.name && i.rarity == driver_def.rarity.db_key())
                                                    .map(|i| i.level)
                                                    .unwrap_or(0);
                                                tr {
                                                    td class=(driver_def.rarity.css_class()) { (driver_def.name) }
                                                    td { (driver_def.rarity.label()) }
                                                    td { (driver_def.series) }
                                                    td {
                                                        select name={"driver:" (driver_def.name) ":" (driver_def.rarity.db_key())} class="inline-select" {
                                                            option value="0" selected[current_level == 0] { "—" }
                                                            @for l in driver_def.levels {
                                                                option value=(l.level) selected[l.level == current_level] {
                                                                    (l.level)
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                button type="submit" { "Save Drivers" }
            }
        },
    )
}
