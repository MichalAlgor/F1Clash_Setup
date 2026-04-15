use maud::{html, Markup};

use crate::auth::AuthStatus;
use crate::models::part::{OwnedPartDefinition, PartCategory};
use crate::models::setup::InventoryItem;

pub fn list_page(items: &[InventoryItem], catalog: &[OwnedPartDefinition], auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Inventory",
        auth,
        html! {
            hgroup {
                h1 { "Parts Inventory" }
                p { "Parts you own and their current levels" }
            }

            a href="/inventory/bulk" role="button" { "Manage All Parts" }

            div class="category-grid" {
                @for category in PartCategory::all() {
                    @let cat_items: Vec<_> = {
                        let mut v: Vec<_> = items.iter()
                            .filter(|item| {
                                catalog.iter()
                                    .find(|p| p.name == item.part_name)
                                    .is_some_and(|p| p.category == *category)
                            })
                            .collect();
                        v.sort_by_key(|item| {
                            catalog.iter().position(|p| p.name == item.part_name).unwrap_or(usize::MAX)
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
                                            @if let Some(part_def) = catalog.iter().find(|p| p.name == item.part_name) {
                                                @if let Some(stats) = part_def.stats_for_level(item.level) {
                                                    tr {
                                                        td class=(part_def.rarity_css_class()) { (item.part_name) }
                                                        td { (part_def.series) }
                                                        td {
                                                            form method="post" action={"/inventory/" (item.id) "/level"} style="display:inline;margin:0" {
                                                                select name="level" onchange="this.form.submit()" class="inline-select" {
                                                                    @for l in &part_def.levels {
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
                }
            }
        },
    )
}

pub fn bulk_page(current_inventory: &[InventoryItem], catalog: &[OwnedPartDefinition], auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Manage All Parts",
        auth,
        html! {
            h1 { "Manage All Parts" }
            p { "Set the level for each part you own. Set to 0 (or leave at \"—\") for parts you don't have." }

            form method="post" action="/inventory/bulk" {
                div class="category-grid" {
                @for category in PartCategory::all() {
                    @let parts: Vec<_> = catalog.iter().filter(|p| p.category == *category).collect();
                    @if !parts.is_empty() {
                        section {
                        h2 { (category.display_name()) }
                        figure {
                            table class="bulk-table" {
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
                                            td class=(part_def.rarity_css_class()) { (part_def.name) }
                                            td { (part_def.series) }
                                            td {
                                                select name={"part:" (part_def.name)} class="inline-select" {
                                                    option value="0" selected[current_level == 0] { "—" }
                                                    @for l in &part_def.levels {
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

                button type="submit" { "Save Inventory" }
            }
        },
    )
}
