use maud::{html, Markup};

use crate::data;
use crate::models::part::PartCategory;
use crate::models::setup::InventoryItem;

pub fn list_page(items: &[InventoryItem]) -> Markup {
    super::layout::page(
        "Inventory",
        html! {
            hgroup {
                h1 { "Parts Inventory" }
                p { "Parts you own and their current levels" }
            }

            a href="/inventory/bulk" role="button" { "Manage All Parts" }

            @for category in PartCategory::all() {
                @let cat_items: Vec<_> = items.iter()
                    .filter(|item| {
                        data::find_part(&item.part_name)
                            .is_some_and(|p| p.category == *category)
                    })
                    .collect();

                @if !cat_items.is_empty() {
                    h2 { (category.display_name()) }
                    figure {
                        table {
                            thead {
                                tr {
                                    th { "Name" }
                                    th { "Series" }
                                    th { "Lvl" }
                                    th { "SPD" }
                                    th { "COR" }
                                    th { "PWR" }
                                    th { "QUA" }
                                    th { "PIT (s)" }
                                    th { "Total" }
                                    th {}
                                }
                            }
                            tbody {
                                @for item in &cat_items {
                                    @if let Some(part_def) = data::find_part(&item.part_name) {
                                        @if let Some(stats) = part_def.stats_for_level(item.level) {
                                            tr {
                                                td { (item.part_name) }
                                                td { (part_def.series) }
                                                td {
                                                    form method="post" action={"/inventory/" (item.id) "/level"} style="display:inline;margin:0" {
                                                        select name="level" onchange="this.form.submit()" style="margin:0;padding:2px 8px;width:auto" {
                                                            @for l in part_def.levels {
                                                                option value=(l.level) selected[l.level == item.level] {
                                                                    (l.level)
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                td { (stats.speed) }
                                                td { (stats.cornering) }
                                                td { (stats.power_unit) }
                                                td { (stats.qualifying) }
                                                td { (format!("{:.2}", stats.pit_stop_time)) }
                                                td { strong { (stats.speed + stats.cornering + stats.power_unit + stats.qualifying) } }
                                                td {
                                                    button.outline.secondary
                                                        hx-delete={"/inventory/" (item.id)}
                                                        hx-confirm="Remove this part?"
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
        },
    )
}

/// Bulk edit page: shows every catalog part with a level selector (0 = not owned)
pub fn bulk_page(current_inventory: &[InventoryItem]) -> Markup {
    super::layout::page(
        "Manage All Parts",
        html! {
            h1 { "Manage All Parts" }
            p { "Set the level for each part you own. Set to 0 (or leave at \"—\") for parts you don't have." }

            form method="post" action="/inventory/bulk" {
                @for category in PartCategory::all() {
                    @let parts = data::parts_by_category(*category);
                    @if !parts.is_empty() {
                        h2 { (category.display_name()) }
                        figure {
                            table {
                                thead {
                                    tr {
                                        th { "Name" }
                                        th { "Series" }
                                        th { "Level" }
                                    }
                                }
                                tbody {
                                    @for part_def in &parts {
                                        @let current_level = current_inventory.iter()
                                            .find(|i| i.part_name == part_def.name)
                                            .map(|i| i.level)
                                            .unwrap_or(0);
                                        tr {
                                            td { (part_def.name) }
                                            td { (part_def.series) }
                                            td {
                                                select name={"part:" (part_def.name)} style="margin:0;padding:2px 8px;width:auto" {
                                                    option value="0" selected[current_level == 0] { "—" }
                                                    @for l in part_def.levels {
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

                button type="submit" { "Save Inventory" }
            }
        },
    )
}
