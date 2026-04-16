use maud::{html, Markup};

use crate::auth::AuthStatus;
use crate::data;
use crate::drivers_data::{self, DriverCategory, DriverDefinition, DriverRarity};
use crate::models::driver::DriverInventoryItem;

pub fn list_page(items: &[DriverInventoryItem], auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Drivers",
        auth,
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
                                            th { "Cards" }
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
                                                        (driver_cards_cell(item.id, item.cards_owned, item.level, Some(driver_def)))
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

pub fn bulk_page(current_inventory: &[DriverInventoryItem], auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Manage All Drivers",
        auth,
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

/// Reactive cards + upgrade cell for the driver inventory list.
pub fn driver_cards_cell(
    item_id: i32,
    cards_owned: i32,
    current_level: i32,
    def: Option<&DriverDefinition>,
) -> Markup {
    let upgrade_markup = match def {
        None => html! {},
        Some(d) => {
            let max_lvl = d.max_level();
            if current_level >= max_lvl {
                html! { span class="upgrade-tag secondary" { "MAX" } }
            } else if cards_owned == 0 {
                html! {}
            } else {
                let (reachable, cards_to_next) =
                    data::calculate_upgrade_cards_only(current_level, cards_owned, max_lvl);
                if reachable > current_level {
                    html! { span class="upgrade-tag" { "→ L" (reachable) } }
                } else {
                    html! {
                        span class="upgrade-tag secondary" title={"Need " (cards_to_next) " more cards"} {
                            "+" (cards_to_next)
                        }
                    }
                }
            }
        }
    };

    html! {
        td id={"dcards-" (item_id)} {
            div class="cards-cell" {
                input type="number" name="cards"
                    value=(cards_owned)
                    min="0"
                    class="cards-input"
                    hx-post={"/drivers/" (item_id) "/cards"}
                    hx-trigger="change"
                    hx-target={"#dcards-" (item_id)}
                    hx-swap="outerHTML"
                    hx-include="this";
                (upgrade_markup)
            }
        }
    }
}
