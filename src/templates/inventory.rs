use maud::{Markup, html};

use crate::auth::AuthStatus;
use crate::data;
use crate::models::part::{OwnedPartDefinition, PartCategory};
use crate::models::setup::InventoryItem;

pub fn list_page(
    items: &[InventoryItem],
    catalog: &[OwnedPartDefinition],
    categories: &[PartCategory],
    auth: &AuthStatus,
    season: &str,
) -> Markup {
    super::layout::page(
        "Inventory",
        auth,
        html! {
            hgroup {
                h1 { "Parts Inventory" }
                p { "Parts you own and their current levels" }
            }

            a href="/inventory/bulk" role="button" { "Manage All Parts" }

            @if items.is_empty() {
                article style="margin-top:1.5rem" {
                    hgroup {
                        h2 { "Welcome! Start by adding your parts." }
                        p { "Your inventory is empty — the optimizer needs parts to work with." }
                    }
                    ol {
                        li {
                            strong { "Add your parts" }
                            " — Use the Bulk Parts form to record which parts you own and at what level."
                        }
                        li {
                            strong { "Run the Optimizer" }
                            " — Pick a race focus (Speed, Cornering, Power Unit) and find the best combination."
                        }
                        li {
                            strong { "Save and compare" }
                            " — Name your setup, save it, and compare configurations side-by-side."
                        }
                    }
                    div style="display:flex;gap:0.75rem;flex-wrap:wrap;margin-top:1rem" {
                        a href="/inventory/bulk" role="button" { "Add your parts →" }
                        a href="/guide" role="button" class="outline" { "Full guide" }
                    }
                }
            } @else {
                div class="category-grid" {
                @for category in categories {
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

                    @let additional_stat_name: Option<String> = catalog.iter()
                        .find(|p| p.category == *category && p.additional_stat_name.is_some())
                        .and_then(|p| p.additional_stat_name.clone());

                    @if !cat_items.is_empty() {
                        section {
                            h2 { img src=(category.icon_path()) class="cat-icon" alt=""; (category.display_name()) }
                            figure {
                                table.responsive-table {
                                    thead {
                                        tr {
                                            th { "Name" }
                                            th { "Series" }
                                            th { "Lvl" }
                                            th { "SPD" }
                                            th { "COR" }
                                            th { "PWR" }
                                            th { "QUA" }
                                            th { "PIT" }
                                            th { "Tot" }
                                            @if let Some(ref stat_name) = additional_stat_name {
                                                th { (stat_name) }
                                            }
                                            th { "Cards" }
                                            th {}
                                        }
                                    }
                                    tbody {
                                        @for item in &cat_items {
                                            @if let Some(part_def) = catalog.iter().find(|p| p.name == item.part_name) {
                                                @if let Some(stats) = part_def.stats_for_level(item.level) {
                                                    tr {
                                                        td { span class=(part_def.rarity_css_class()) { (item.part_name) } }
                                                        td data-label="Series" { (part_def.series) }
                                                        td data-label="Lvl" {
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
                                                        td.stat-cell data-label="SPD" { (stats.speed) }
                                                        td.stat-cell data-label="COR" { (stats.cornering) }
                                                        td.stat-cell data-label="PWR" { (stats.power_unit) }
                                                        td.stat-cell data-label="QUA" { (stats.qualifying) }
                                                        td.stat-cell data-label="PIT" { (format!("{:.2}", stats.pit_stop_time)) }
                                                        td.stat-cell data-label="Total" { strong { (stats.speed + stats.cornering + stats.power_unit + stats.qualifying) } }
                                                        @if additional_stat_name.is_some() {
                                                            td data-label="Special" {
                                                                @if stats.additional_stat_value > 0 {
                                                                    (stats.additional_stat_value)
                                                                    @if !stats.additional_stat_details.is_empty() {
                                                                        @let sub: Vec<_> = {
                                                                            let mut v: Vec<_> = stats.additional_stat_details.iter().collect();
                                                                            v.sort_by_key(|(k, _)| k.as_str());
                                                                            v
                                                                        };
                                                                        br;
                                                                        small class="secondary" {
                                                                            @for (i, (key, val)) in sub.iter().enumerate() {
                                                                                @if i > 0 { " · " }
                                                                                (key) ": " (val)
                                                                            }
                                                                        }
                                                                    }
                                                                } @else { "—" }
                                                            }
                                                        }
                                                        (cards_cell(item.id, item.cards_owned, item.level, Some(part_def), season))
                                                        td.action-cell {
                                                            button.outline.secondary
                                                                hx-post={"/inventory/" (item.id) "/delete"}
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
            }
        },
    )
}

/// The reactive cards + upgrade cell. Returned by both the list page and the
/// `POST /inventory/{id}/cards` endpoint so htmx can swap it in place.
pub fn cards_cell(
    item_id: i32,
    cards_owned: i32,
    current_level: i32,
    part_def: Option<&OwnedPartDefinition>,
    season: &str,
) -> Markup {
    let upgrade_markup = match part_def {
        None => html! {},
        Some(part) => {
            let max_lvl = data::max_level_for_rarity(&part.rarity);
            if current_level >= max_lvl {
                html! { span class="upgrade-tag secondary" { "MAX" } }
            } else if cards_owned == 0 {
                html! {}
            } else {
                let upgrade = data::calculate_upgrade(
                    current_level,
                    cards_owned,
                    part.series,
                    &part.rarity,
                    season,
                );
                if upgrade.reachable_level > current_level {
                    html! {
                        span class="upgrade-tag" title={"Coins: " (upgrade.coins_needed)} {
                            "→ L" (upgrade.reachable_level) " · " (data::format_coins(upgrade.coins_needed))
                        }
                    }
                } else {
                    html! {
                        span class="upgrade-tag secondary" title={"Need " (upgrade.cards_to_next) " more cards"} {
                            "+" (upgrade.cards_to_next)
                        }
                    }
                }
            }
        }
    };

    html! {
        td id={"cards-" (item_id)} data-label="Cards" {
            div class="cards-cell" {
                input type="number" name="cards"
                    value=(cards_owned)
                    min="0"
                    class="cards-input"
                    hx-post={"/inventory/" (item_id) "/cards"}
                    hx-trigger="change"
                    hx-target={"#cards-" (item_id)}
                    hx-swap="outerHTML"
                    hx-include="this";
                (upgrade_markup)
            }
        }
    }
}

/// Bulk edit page: shows every catalog part with a level selector (0 = not owned)
pub fn bulk_page(
    current_inventory: &[InventoryItem],
    catalog: &[OwnedPartDefinition],
    categories: &[PartCategory],
    auth: &AuthStatus,
) -> Markup {
    super::layout::page(
        "Manage All Parts",
        auth,
        html! {
            h1 { "Manage All Parts" }
            p { "Set the level for each part you own. Set to 0 (or leave at \"—\") for parts you don't have." }

            form method="post" action="/inventory/bulk" {
                div class="category-grid" {
                @for category in categories {
                    @let parts: Vec<_> = catalog.iter().filter(|p| p.category == *category).collect();
                    @if !parts.is_empty() {
                        section {
                        h2 { img src=(category.icon_path()) class="cat-icon" alt=""; (category.display_name()) }
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
