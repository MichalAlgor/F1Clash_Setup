use maud::{html, Markup};

use crate::models::part::Part;
use crate::models::setup::SetupWithStats;

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
                p { "No setups yet. Create one to get started!" }
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

pub fn form_page(parts_by_category: &[(String, Vec<Part>)], setup: Option<&crate::models::setup::Setup>) -> Markup {
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

                @for (category_name, parts) in parts_by_category {
                    label for=(category_name) { (category_name) }
                    select id=(category_name) name=(category_name) required {
                        option value="" { "Select a part…" }
                        @for part in parts {
                            option value=(part.id) {
                                (part.name) " (Lvl " (part.level) " — " (part.stats().total_performance()) " perf)"
                            }
                        }
                    }
                }

                button type="submit" { "Save Setup" }
            }
        },
    )
}
