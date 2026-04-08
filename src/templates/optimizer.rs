use maud::{html, Markup};

use crate::data::StatPriorities;
use crate::models::part::{PartCategory, Stats};
use crate::models::setup::InventoryItem;

pub fn form_page() -> Markup {
    super::layout::page(
        "Optimizer",
        html! {
            hgroup {
                h1 { "Setup Optimizer" }
                p { "Select stats to prioritize, then find the best setup from your inventory" }
            }

            form method="get" action="/optimizer/run" {
                fieldset {
                    legend { "Prioritize stats" }
                    label {
                        input type="checkbox" name="speed" value="true";
                        " Speed"
                    }
                    label {
                        input type="checkbox" name="cornering" value="true";
                        " Cornering"
                    }
                    label {
                        input type="checkbox" name="power_unit" value="true";
                        " Power Unit"
                    }
                    label {
                        input type="checkbox" name="qualifying" value="true";
                        " Qualifying"
                    }
                }

                button type="submit" { "Find Best Setup" }
            }
        },
    )
}

pub fn result_page(
    priorities: &StatPriorities,
    picks: &[(PartCategory, InventoryItem, Stats)],
    total: &Stats,
) -> Markup {
    let priority_labels = priorities.labels().join(", ");

    super::layout::page(
        "Optimizer Result",
        html! {
            h1 { "Optimized Setup" }
            @if priorities.any_selected() {
                p { "Prioritizing: " strong { (priority_labels) } }
            } @else {
                p { "No priorities selected — optimizing for highest total performance" }
            }

            @if picks.is_empty() {
                p { "No parts in inventory. Add parts first!" }
            } @else {
                figure {
                    table {
                        thead {
                            tr {
                                th { "Category" }
                                th { "Part" }
                                th { "Lvl" }
                                th { "SPD" }
                                th { "COR" }
                                th { "PWR" }
                                th { "QUA" }
                                th { "PIT (s)" }
                                th { "Total" }
                            }
                        }
                        tbody {
                            @for (cat, item, stats) in picks {
                                tr {
                                    td { (cat.display_name()) }
                                    td { strong { (item.part_name.clone()) } }
                                    td { (item.level) }
                                    td { (stats.speed) }
                                    td { (stats.cornering) }
                                    td { (stats.power_unit) }
                                    td { (stats.qualifying) }
                                    td { (format!("{:.2}", stats.pit_stop_time)) }
                                    td { (stats.total_performance()) }
                                }
                            }
                        }
                        tfoot {
                            tr {
                                td colspan="3" { strong { "Total" } }
                                td { strong { (total.speed) } }
                                td { strong { (total.cornering) } }
                                td { strong { (total.power_unit) } }
                                td { strong { (total.qualifying) } }
                                td { strong { (format!("{:.2}", total.pit_stop_time)) } }
                                td { strong { (total.total_performance()) } }
                            }
                        }
                    }
                }

                // Save as setup form
                h2 { "Save this setup" }
                form method="post" action="/optimizer/save" {
                    label for="name" { "Setup Name" }
                    input type="text" id="name" name="name" required
                        value=(format!("Optimized ({})", if priority_labels.is_empty() { "Total".to_string() } else { priority_labels }));

                    @for (cat, item, _stats) in picks {
                        input type="hidden" name=(format!("{}_id", cat.slug())) value=(item.id);
                    }

                    button type="submit" { "Save Setup" }
                }

                a href="/optimizer" role="button" class="outline" { "← Try different priorities" }
            }
        },
    )
}
