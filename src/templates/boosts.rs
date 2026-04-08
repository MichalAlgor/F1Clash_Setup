use maud::{html, Markup, PreEscaped};

use crate::data;
use crate::models::part::PartCategory;
use crate::models::setup::Boost;

pub fn page(boosts: &[Boost]) -> Markup {
    // Collect names of currently boosted parts for JS init
    let boosted_names: Vec<&str> = boosts.iter().map(|b| b.part_name.as_str()).collect();

    super::layout::page(
        "Boosts",
        html! {
            hgroup {
                h1 { "Global Boosts" }
                p { "Check parts that have a boost, then set the percentage" }
            }

            form method="post" action="/boosts" id="boost-form" {
                // Active boosts section
                div id="active-boosts" {
                    @if boosts.is_empty() {
                        p.secondary #no-boosts { "No boosts active. Select parts below." }
                    }
                    @for boost in boosts {
                        @if let Some(part_def) = data::find_part(&boost.part_name) {
                            div class="boost-entry" data-part=(boost.part_name) {
                                span class="boost-name" { (boost.part_name) }
                                span class="boost-cat" { (part_def.category.display_name()) }
                                input type="number"
                                    name={"boost:" (boost.part_name)}
                                    min="1" max="100" step="1"
                                    value=(boost.percentage)
                                    class="compact";
                                span { "%" }
                                button type="button" class="btn-delete outline"
                                    onclick={"toggleBoost('" (boost.part_name) "', false)"} { "×" }
                            }
                        }
                    }
                }

                button type="submit" { "Save Boosts" }

                // Part selection by category
                div class="category-grid" {
                    @for category in PartCategory::all() {
                        @let parts = data::parts_by_category(*category);
                        @if !parts.is_empty() {
                            section {
                                h2 { (category.display_name()) }
                                figure {
                                    table {
                                        thead {
                                            tr {
                                                th {}
                                                th { "Part" }
                                                th { "Series" }
                                            }
                                        }
                                        tbody {
                                            @for part_def in &parts {
                                                @let is_boosted = boosted_names.contains(&part_def.name);
                                                tr {
                                                    td {
                                                        input type="checkbox"
                                                            class="boost-check"
                                                            data-part=(part_def.name)
                                                            data-category=(category.display_name())
                                                            checked[is_boosted]
                                                            onchange={"toggleBoost('" (part_def.name) "', this.checked, '" (category.display_name()) "')"};
                                                    }
                                                    td class=(part_def.rarity.css_class()) { (part_def.name) }
                                                    td { (part_def.series) }
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

            (PreEscaped(BOOST_JS))
        },
    )
}

const BOOST_JS: &str = r#"
<script>
function toggleBoost(partName, enabled, category) {
    const container = document.getElementById('active-boosts');
    const noBoosts = document.getElementById('no-boosts');
    const existing = container.querySelector(`[data-part="${partName}"]`);

    if (enabled && !existing) {
        if (noBoosts) noBoosts.remove();
        const entry = document.createElement('div');
        entry.className = 'boost-entry';
        entry.dataset.part = partName;
        entry.innerHTML = `
            <span class="boost-name">${partName}</span>
            <span class="boost-cat">${category || ''}</span>
            <input type="number" name="boost:${partName}" min="1" max="100" step="1" value="10" class="compact">
            <span>%</span>
            <button type="button" class="btn-delete outline" onclick="toggleBoost('${partName}', false)">×</button>
        `;
        container.appendChild(entry);
    } else if (!enabled && existing) {
        existing.remove();
        // Uncheck the checkbox
        const cb = document.querySelector(`.boost-check[data-part="${partName}"]`);
        if (cb) cb.checked = false;
        // Show "no boosts" if empty
        if (!container.querySelector('.boost-entry')) {
            const p = document.createElement('p');
            p.className = 'secondary';
            p.id = 'no-boosts';
            p.textContent = 'No boosts active. Select parts below.';
            container.appendChild(p);
        }
    }
}
</script>
"#;
