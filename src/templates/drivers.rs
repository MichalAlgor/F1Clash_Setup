use maud::{Markup, html};

use crate::auth::AuthStatus;
use crate::data;
use crate::drivers_data::{DriverCategory, DriverRarity};
use crate::models::driver::{DriverInventoryItem, OwnedDriverDefinition};

pub fn list_page(
    items: &[DriverInventoryItem],
    catalog: &[OwnedDriverDefinition],
    auth: &AuthStatus,
) -> Markup {
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
                                catalog.iter().find(|d| d.name == item.driver_name && d.rarity == item.rarity)
                                    .and_then(|d| DriverRarity::from_db(&d.rarity))
                                    .is_some_and(|r| r.category() == *category)
                            })
                            .collect();
                        v.sort_by_key(|item| {
                            catalog.iter()
                                .find(|d| d.name == item.driver_name && d.rarity == item.rarity)
                                .map_or(i32::MAX, |d| d.sort_order)
                        });
                        v
                    };

                    @if !cat_items.is_empty() {
                        section {
                            h2 { (category.display_name()) }
                            figure {
                                table.responsive-table {
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
                                            @if let Some(driver_def) = catalog.iter().find(|d| d.name == item.driver_name && d.rarity == item.rarity) {
                                                @if let Some(stats) = driver_def.stats_for_level(item.level) {
                                                    @let rarity_css = DriverRarity::from_db(&driver_def.rarity).map_or("", |r| r.css_class());
                                                    tr {
                                                        td class=(rarity_css) { (item.driver_name) }
                                                        td data-label="Rarity" { (driver_def.rarity) }
                                                        td data-label="Series" { (driver_def.series) }
                                                        td data-label="Lvl" {
                                                            form method="post" action={"/drivers/" (item.id) "/level"} style="display:inline;margin:0" {
                                                                select name="level" onchange="this.form.submit()" class="inline-select" {
                                                                    @for l in &driver_def.levels {
                                                                        option value=(l.level) selected[l.level == item.level] {
                                                                            (l.level)
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        td.stat-cell data-label="OVT" { (stats.overtaking) }
                                                        td.stat-cell data-label="DEF" { (stats.defending) }
                                                        td.stat-cell data-label="QUA" { (stats.qualifying) }
                                                        td.stat-cell data-label="RST" { (stats.race_start) }
                                                        td.stat-cell data-label="TYR" { (stats.tyre_management) }
                                                        td.stat-cell data-label="Total" { strong { (stats.total()) } }
                                                        (driver_cards_cell(item.id, item.cards_owned, item.level, Some(driver_def)))
                                                        td.action-cell {
                                                            button.outline.secondary
                                                                hx-post={"/drivers/" (item.id) "/delete"}
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

pub fn bulk_page(
    current_inventory: &[DriverInventoryItem],
    catalog: &[OwnedDriverDefinition],
    auth: &AuthStatus,
) -> Markup {
    super::layout::page(
        "Manage All Drivers",
        auth,
        html! {
            h1 { "Manage All Drivers" }
            p { "Set the level for each driver you own. Leave at \"—\" for drivers you don't have." }

            form method="post" action="/drivers/bulk" {
                div class="category-grid" {
                    @for category in DriverCategory::all() {
                        @let drivers: Vec<_> = catalog.iter()
                            .filter(|d| DriverRarity::from_db(&d.rarity)
                                .is_some_and(|r| r.category() == *category))
                            .collect();
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
                                                    .find(|i| i.driver_name == driver_def.name && i.rarity == driver_def.rarity)
                                                    .map(|i| i.level)
                                                    .unwrap_or(0);
                                                @let rarity_css = DriverRarity::from_db(&driver_def.rarity).map_or("", |r| r.css_class());
                                                tr {
                                                    td class=(rarity_css) { (driver_def.name) }
                                                    td { (driver_def.rarity) }
                                                    td { (driver_def.series) }
                                                    td {
                                                        select name={"driver:" (driver_def.name) ":" (driver_def.rarity)} class="inline-select" {
                                                            option value="0" selected[current_level == 0] { "—" }
                                                            @for l in &driver_def.levels {
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
    def: Option<&OwnedDriverDefinition>,
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
        td id={"dcards-" (item_id)} data-label="Cards" {
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
