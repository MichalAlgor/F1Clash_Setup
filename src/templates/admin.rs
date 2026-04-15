use std::collections::HashMap;

use maud::{html, Markup};

use crate::auth::AuthStatus;
use crate::models::part::{PartCategory, OwnedPartDefinition};

pub fn parts_list_page(catalog: &[OwnedPartDefinition], active_season: &str, auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Admin — Parts",
        auth,
        html! {
            hgroup {
                h1 { "Admin: Parts Catalog" }
                p { "Season: " strong { (active_season) } }
            }

            div style="display:flex;gap:1rem;flex-wrap:wrap;margin-bottom:1rem" {
                a href="/admin/parts/new" role="button" { "+ Add Part" }
                a href="/admin/parts/export" role="button" class="outline" { "Export parts.json" }
                a href="/admin/seasons" role="button" class="outline" { "Season Settings" }
            }

            @if catalog.is_empty() {
                p { "No parts for this season yet." }
            } @else {
                div class="category-grid" {
                    @for category in PartCategory::all() {
                        @let parts: Vec<_> = catalog.iter().filter(|p| p.category == *category).collect();
                        @if !parts.is_empty() {
                            section {
                                h2 { (category.display_name()) }
                                figure {
                                    table {
                                        thead {
                                            tr {
                                                th { "Name" }
                                                th { "Series" }
                                                th { "Rarity" }
                                                th { "Levels" }
                                                th { "Special Stat" }
                                                th {}
                                            }
                                        }
                                        tbody {
                                            @for part in &parts {
                                                tr {
                                                    td class=(part.rarity_css_class()) { (part.name) }
                                                    td { (part.series) }
                                                    td { (part.rarity) }
                                                    td { (part.levels.len()) }
                                                    td {
                                                        @if let Some(ref name) = part.additional_stat_name {
                                                            (name)
                                                        } @else {
                                                            span class="secondary" { "—" }
                                                        }
                                                    }
                                                    td style="white-space:nowrap" {
                                                        a href={"/admin/parts/" (part.id) "/edit"} role="button" class="outline" style="padding:0.2rem 0.5rem;margin-right:0.3rem" { "Edit" }
                                                        button
                                                            hx-delete={"/admin/parts/" (part.id)}
                                                            hx-confirm={"Delete \"" (part.name) "\"?"}
                                                            hx-target="closest tr"
                                                            hx-swap="outerHTML"
                                                            class="outline secondary"
                                                            style="padding:0.2rem 0.5rem"
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
        },
    )
}

pub fn part_form_page(part: Option<&OwnedPartDefinition>, active_season: &str, auth: &AuthStatus) -> Markup {
    let title = if part.is_some() { "Edit Part" } else { "New Part" };
    let action = match part {
        Some(p) => format!("/admin/parts/{}", p.id),
        None => "/admin/parts".to_string(),
    };

    let levels_json = match part {
        Some(p) => serde_json::to_string_pretty(&p.levels).unwrap_or_default(),
        None => serde_json::to_string_pretty(&serde_json::json!([
            {
                "level": 1, "speed": 0, "cornering": 0, "power_unit": 0, "qualifying": 0,
                "pit_stop_time": 1.0, "additional_stat_value": 0, "additional_stat_details": {}
            }
        ])).unwrap_or_default(),
    };

    super::layout::page(
        title,
        auth,
        html! {
            hgroup {
                h1 { (title) }
                p { "Season: " strong { (active_season) } }
            }

            form method="post" action=(action) {
                label for="name" { "Name" }
                input type="text" id="name" name="name" required
                    value=[part.map(|p| p.name.as_str())];

                label for="category" { "Category" }
                select id="category" name="category" required {
                    @for cat in PartCategory::all() {
                        option value=(cat.slug()) selected[part.is_some_and(|p| p.category == *cat)] {
                            (cat.display_name())
                        }
                    }
                }

                label for="series" { "Series" }
                input type="number" id="series" name="series" min="1" required
                    value=[part.map(|p| p.series)];

                label for="rarity" { "Rarity" }
                select id="rarity" name="rarity" required {
                    @for r in &["Common", "Rare", "Epic"] {
                        option value=(r) selected[part.is_some_and(|p| p.rarity == *r)] {
                            (r)
                        }
                    }
                }

                label for="additional_stat_name" {
                    "Special Stat Name "
                    small { "(optional — e.g. \"DRS\", \"Overtake Mode\"; leave blank if none)" }
                }
                input type="text" id="additional_stat_name" name="additional_stat_name"
                    placeholder="e.g. Overtake Mode"
                    value=[part.and_then(|p| p.additional_stat_name.as_deref())];

                label for="levels_json" {
                    "Level Stats (JSON array)"
                    small {
                        " — fields: level, speed, cornering, power_unit, qualifying, pit_stop_time"
                        ", additional_stat_value, additional_stat_details"
                    }
                }
                textarea id="levels_json" name="levels_json" rows="15" required
                    style="font-family:monospace;font-size:0.85em" {
                    (levels_json)
                }

                div style="display:flex;gap:1rem" {
                    button type="submit" { "Save" }
                    a href="/admin/parts" role="button" class="outline secondary" { "Cancel" }
                }
            }
        },
    )
}

pub fn seasons_page(
    season_cats: &HashMap<String, Vec<PartCategory>>,
    active_season: &str,
    auth: &AuthStatus,
) -> Markup {
    // Collect all known seasons (from the map + active season)
    let mut all_seasons: Vec<String> = season_cats.keys().cloned().collect();
    if !all_seasons.contains(&active_season.to_string()) {
        all_seasons.push(active_season.to_string());
    }
    all_seasons.sort();

    let all_categories = PartCategory::all();

    super::layout::page(
        "Admin — Season Settings",
        auth,
        html! {
            hgroup {
                h1 { "Season Settings" }
                p { "Configure which part categories are active for each season." }
            }

            p {
                small class="secondary" {
                    "These define which part slots a setup requires and which categories appear in the optimizer."
                }
            }

            @for season in &all_seasons {
                @let enabled: Vec<PartCategory> = season_cats.get(season).cloned().unwrap_or_default();

                details open[season == active_season] {
                    summary { strong { (season) } @if season == active_season { " (active)" } }

                    form method="post" action="/admin/seasons" style="margin-top:0.75rem" {
                        input type="hidden" name="season" value=(season);

                        div style="display:flex;flex-wrap:wrap;gap:0.75rem;margin-bottom:1rem" {
                            @for cat in all_categories {
                                label style="display:flex;align-items:center;gap:0.4rem;cursor:pointer" {
                                    input type="checkbox"
                                        name="categories"
                                        value=(cat.slug())
                                        checked[enabled.contains(cat)];
                                    (cat.display_name())
                                }
                            }
                        }

                        button type="submit" class="outline" style="padding:0.3rem 0.8rem;font-size:0.85rem" {
                            "Save " (season)
                        }
                    }
                }
            }

            hr;

            h2 { "Add new season" }
            form method="post" action="/admin/seasons" {
                div style="display:flex;gap:1rem;align-items:flex-end;flex-wrap:wrap" {
                    div {
                        label for="new_season_name" { "Season name" }
                        input type="hidden" name="season" id="new_season_name_hidden";
                        input type="text" id="new_season_name" placeholder="e.g. 2027"
                            oninput="document.getElementById('new_season_name_hidden').value=this.value"
                            style="width:120px";
                    }
                    div style="flex:1;min-width:300px" {
                        label { "Categories" }
                        div style="display:flex;flex-wrap:wrap;gap:0.75rem" {
                            @for cat in all_categories {
                                label style="display:flex;align-items:center;gap:0.4rem;cursor:pointer" {
                                    input type="checkbox" name="categories" value=(cat.slug());
                                    (cat.display_name())
                                }
                            }
                        }
                    }
                    button type="submit" { "Create Season" }
                }
            }
        },
    )
}
