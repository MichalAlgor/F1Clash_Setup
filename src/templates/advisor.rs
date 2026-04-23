use maud::{Markup, html};

use crate::auth::AuthStatus;
use crate::upgrade_advisor::AdvisorResult;

pub fn form_page(auth: &AuthStatus) -> Markup {
    super::layout::page(
        "Upgrade Advisor",
        auth,
        html! {
            hgroup {
                h1 { "Upgrade Advisor" }
                p { "Find which parts to upgrade for the biggest score improvement" }
            }

            form method="get" action="/advisor/run" {
                fieldset {
                    legend { "Part stat priorities" }
                    p class="secondary" style="font-size:0.85rem;margin-bottom:0.5rem" {
                        "Select the stats you care about. Leave blank to optimise for highest total."
                    }
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

                fieldset {
                    legend { "Series limit" }
                    div class="series-limits-grid" {
                        label {
                            "Max part series (1–12)"
                            input type="number" name="max_part_series" min="1" max="12" value="12";
                        }
                    }
                }

                button type="submit" { "Analyse Upgrades" }
            }
        },
    )
}

pub fn result_page(result: &AdvisorResult, auth: &AuthStatus) -> Markup {
    let priority_label = {
        let labels = result.priorities.labels();
        if labels.is_empty() {
            "Total performance".to_string()
        } else {
            labels.join(", ")
        }
    };

    super::layout::page(
        "Upgrade Advisor — Results",
        auth,
        html! {
            hgroup {
                h1 { "Upgrade Advisor" }
                p {
                    "Baseline score: " strong { (result.baseline_score) }
                    " · Priority: " strong { (priority_label) }
                }
            }

            a href="/advisor" role="button" class="outline back-link" { "← Change priorities" }

            // ── Section 1: Ready to Upgrade ───────────────────────────────────
            h2 { "Ready to Upgrade" }
            @if result.immediate.is_empty() {
                p class="secondary" {
                    "No upgrades available — you need more cards, or all parts are at max level."
                }
            } @else {
                figure {
                    table.responsive-table {
                        thead {
                            tr {
                                th { "Part" }
                                th { "Category" }
                                th { "Level" }
                                th { "Score Impact" }
                                th { "Coins" }
                                th { "Stat Changes" }
                            }
                        }
                        tbody {
                            @for rec in &result.immediate {
                                tr {
                                    td { span class=(rec.candidate.rarity_css_class) { (rec.candidate.part_name) } }
                                    td data-label="Category" {
                                        (rec.candidate.category.display_name())
                                    }
                                    td data-label="Level" {
                                        (rec.candidate.current_level) " → " (rec.candidate.target_level)
                                    }
                                    td.stat-cell data-label="Impact" {
                                        @if rec.score_delta > 0 {
                                            strong class="upgrade-positive" { "+" (rec.score_delta) }
                                        } @else if rec.score_delta == 0 {
                                            span class="secondary" { "±0" }
                                        } @else {
                                            span class="secondary" { (rec.score_delta) }
                                        }
                                    }
                                    td data-label="Coins" {
                                        (rec.cost.coins_display)
                                    }
                                    td data-label="Stats" {
                                        (stat_delta_display(&rec.stat_delta))
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Section 2: Best to Target ─────────────────────────────────────
            h2 { "Best to Target" }
            p class="secondary" style="font-size:0.85rem;margin-bottom:0.5rem" {
                "All possible +1 upgrades ranked by score impact. Plan which cards to collect next."
            }
            @if result.planned.is_empty() {
                p class="secondary" { "All parts are at max level." }
            } @else {
                figure {
                    table.responsive-table {
                        thead {
                            tr {
                                th { "Part" }
                                th { "Category" }
                                th { "Level" }
                                th { "Score Impact" }
                                th { "Cards Needed" }
                                th { "Coins" }
                                th { "Stat Changes" }
                            }
                        }
                        tbody {
                            @for rec in &result.planned {
                                @let is_ready = rec.cost.can_afford;
                                tr {
                                    td {
                                        span class=(rec.candidate.rarity_css_class) { (rec.candidate.part_name) }
                                        @if is_ready {
                                            " " span class="ready-badge" { "Ready" }
                                        }
                                    }
                                    td data-label="Category" {
                                        (rec.candidate.category.display_name())
                                    }
                                    td data-label="Level" {
                                        (rec.candidate.current_level) " → " (rec.candidate.target_level)
                                    }
                                    td.stat-cell data-label="Impact" {
                                        @if rec.score_delta > 0 {
                                            strong class="upgrade-positive" { "+" (rec.score_delta) }
                                        } @else if rec.score_delta == 0 {
                                            span class="secondary" { "±0" }
                                        } @else {
                                            span class="secondary" { (rec.score_delta) }
                                        }
                                    }
                                    td data-label="Cards" {
                                        @if is_ready {
                                            span class="secondary" { "✓ Have enough" }
                                        } @else {
                                            "+" (rec.cost.cards_needed - rec.cost.cards_owned)
                                        }
                                    }
                                    td data-label="Coins" {
                                        (rec.cost.coins_display)
                                    }
                                    td data-label="Stats" {
                                        (stat_delta_display(&rec.stat_delta))
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

fn stat_delta_display(delta: &crate::upgrade_advisor::StatDelta) -> Markup {
    html! {
        span class="stat-delta" {
            @if delta.speed != 0 { span { "SPD " (fmt_delta(delta.speed)) } " " }
            @if delta.cornering != 0 { span { "COR " (fmt_delta(delta.cornering)) } " " }
            @if delta.power_unit != 0 { span { "PWR " (fmt_delta(delta.power_unit)) } " " }
            @if delta.qualifying != 0 { span { "QUA " (fmt_delta(delta.qualifying)) } " " }
            @if delta.pit_stop_time.abs() > 0.001 {
                span { "PIT " (fmt_delta_f(delta.pit_stop_time)) }
            }
        }
    }
}

fn fmt_delta(v: i32) -> String {
    if v > 0 {
        format!("+{v}")
    } else {
        format!("{v}")
    }
}

fn fmt_delta_f(v: f64) -> String {
    if v > 0.0 {
        format!("+{:.2}", v)
    } else {
        format!("{:.2}", v)
    }
}
