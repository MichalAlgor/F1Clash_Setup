use maud::{html, Markup};

use crate::models::part::{Part, PartCategory};

pub fn list_page(parts: &[Part]) -> Markup {
    super::layout::page(
        "Parts",
        html! {
            hgroup {
                h1 { "Parts Inventory" }
                p { "Manage your car parts collection" }
            }

            a href="/parts/new" role="button" { "Add Part" }

            @for category in PartCategory::all() {
                @let cat_parts: Vec<_> = parts.iter()
                    .filter(|p| p.category == *category)
                    .collect();

                @if !cat_parts.is_empty() {
                    h2 { (category.display_name()) }
                    figure {
                        table {
                            thead {
                                tr {
                                    th { "Name" }
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
                                @for part in &cat_parts {
                                    tr {
                                        td { (part.name) }
                                        td { (part.level) }
                                        td { (part.speed) }
                                        td { (part.cornering) }
                                        td { (part.power_unit) }
                                        td { (part.qualifying) }
                                        td { (format!("{:.2}", part.pit_stop_time)) }
                                        td { strong { (part.stats().total_performance()) } }
                                        td {
                                            a href={"/parts/" (part.id) "/edit"} { "Edit" }
                                            " "
                                            button.outline.secondary
                                                hx-delete={"/parts/" (part.id)}
                                                hx-confirm="Delete this part?"
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
        },
    )
}

pub fn form_page(part: Option<&Part>) -> Markup {
    let title = if part.is_some() { "Edit Part" } else { "Add Part" };
    let action = match part {
        Some(p) => format!("/parts/{}", p.id),
        None => "/parts".to_string(),
    };

    super::layout::page(
        title,
        html! {
            h1 { (title) }
            form method="post" action=(action) {
                label for="name" { "Name" }
                input type="text" id="name" name="name" required
                    value=[part.map(|p| p.name.as_str())];

                label for="category" { "Category" }
                select id="category" name="category" required {
                    @for cat in PartCategory::all() {
                        option
                            value=(format!("{cat:?}"))
                            selected[part.is_some_and(|p| p.category == *cat)]
                        { (cat.display_name()) }
                    }
                }

                label for="level" { "Level" }
                input type="number" id="level" name="level" min="1" max="12" required
                    value=(part.map_or(1, |p| p.level));

                fieldset.grid {
                    div {
                        label for="speed" { "Speed" }
                        input type="number" id="speed" name="speed" required
                            value=(part.map_or(0, |p| p.speed));
                    }
                    div {
                        label for="cornering" { "Cornering" }
                        input type="number" id="cornering" name="cornering" required
                            value=(part.map_or(0, |p| p.cornering));
                    }
                    div {
                        label for="power_unit" { "Power Unit" }
                        input type="number" id="power_unit" name="power_unit" required
                            value=(part.map_or(0, |p| p.power_unit));
                    }
                    div {
                        label for="qualifying" { "Qualifying" }
                        input type="number" id="qualifying" name="qualifying" required
                            value=(part.map_or(0, |p| p.qualifying));
                    }
                    div {
                        label for="pit_stop_time" { "Pit Stop Time (s)" }
                        input type="number" id="pit_stop_time" name="pit_stop_time" step="0.01" required
                            value=(format!("{:.2}", part.map_or(0.0, |p| p.pit_stop_time)));
                    }
                }

                button type="submit" { "Save Part" }
            }
        },
    )
}
