use maud::{html, Markup};

use crate::data;
use crate::models::part::PartCategory;
use crate::models::setup::Boost;

pub fn page(boosts: &[Boost]) -> Markup {
    super::layout::page(
        "Boosts",
        html! {
            hgroup {
                h1 { "Global Boosts" }
                p { "Select parts to boost and their boost percentage" }
            }

            form method="post" action="/boosts" {
                @for category in PartCategory::all() {
                    @let parts = data::parts_by_category(*category);
                    @if !parts.is_empty() {
                        h2 { (category.display_name()) }
                        figure {
                            table {
                                thead {
                                    tr {
                                        th { "Part" }
                                        th { "Series" }
                                        th { "Boost %" }
                                    }
                                }
                                tbody {
                                    @for part_def in &parts {
                                        @let current_pct = boosts.iter()
                                            .find(|b| b.part_name == part_def.name)
                                            .map(|b| b.percentage)
                                            .unwrap_or(0);
                                        tr {
                                            td { (part_def.name) }
                                            td { (part_def.series) }
                                            td {
                                                input type="number"
                                                    name={"boost:" (part_def.name)}
                                                    min="0" max="100" step="1"
                                                    value=(current_pct)
                                                    style="margin:0;padding:2px 8px;width:80px";
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                button type="submit" { "Save Boosts" }
            }
        },
    )
}
