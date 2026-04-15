use maud::{html, Markup};

use crate::auth::AuthStatus;
use crate::models::part::{OwnedPartDefinition, PartCategory};

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
            { "level": 1, "speed": 0, "cornering": 0, "power_unit": 0, "qualifying": 0, "pit_stop_time": 1.0, "drs": 0 }
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

                label for="levels_json" {
                    "Level Stats (JSON array)"
                    small { " — fields: level, speed, cornering, power_unit, qualifying, pit_stop_time, drs" }
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
