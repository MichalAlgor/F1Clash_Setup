use maud::{html, Markup, PreEscaped};

use crate::drivers_data::{self, DriverCategory};
use crate::models::driver::DriverBoost;

pub fn page(boosts: &[DriverBoost]) -> Markup {
    let boosted_keys: Vec<(String, String)> = boosts
        .iter()
        .map(|b| (b.driver_name.clone(), b.rarity.clone()))
        .collect();

    super::layout::page(
        "Driver Boosts",
        html! {
            hgroup {
                h1 { "Driver Boosts" }
                p { "Check drivers that have a boost, then set the percentage" }
            }

            form method="post" action="/driver-boosts" id="driver-boost-form" {
                div id="active-driver-boosts" {
                    @if boosts.is_empty() {
                        p.secondary #no-driver-boosts { "No driver boosts active. Select drivers below." }
                    }
                    @for boost in boosts {
                        @if let Some(driver_def) = drivers_data::find_driver_by_db(&boost.driver_name, &boost.rarity) {
                            div class="boost-entry" data-key={(boost.driver_name) ":" (boost.rarity)} {
                                span class={"boost-name " (driver_def.rarity.css_class())} { (boost.driver_name) }
                                span class="boost-cat" { (driver_def.rarity.label()) }
                                input type="number"
                                    name={"boost:" (boost.driver_name) ":" (boost.rarity)}
                                    min="1" max="100" step="1"
                                    value=(boost.percentage)
                                    class="compact";
                                span { "%" }
                                button type="button" class="btn-delete outline"
                                    onclick={"toggleDriverBoost('" (boost.driver_name) "', '" (boost.rarity) "', false)"} { "×" }
                            }
                        }
                    }
                }

                button type="submit" { "Save Driver Boosts" }

                div class="category-grid" {
                    @for category in DriverCategory::all() {
                        @let drivers = drivers_data::drivers_by_category(*category);
                        @if !drivers.is_empty() {
                            section {
                                h2 { (category.display_name()) }
                                figure {
                                    table {
                                        thead {
                                            tr {
                                                th {}
                                                th { "Driver" }
                                                th { "Rarity" }
                                                th { "Series" }
                                            }
                                        }
                                        tbody {
                                            @for driver_def in &drivers {
                                                @let is_boosted = boosted_keys.iter()
                                                    .any(|(n, r)| n == driver_def.name && r == driver_def.rarity.db_key());
                                                tr {
                                                    td {
                                                        input type="checkbox"
                                                            class="driver-boost-check"
                                                            data-name=(driver_def.name)
                                                            data-rarity=(driver_def.rarity.db_key())
                                                            data-css=(driver_def.rarity.css_class())
                                                            data-label=(driver_def.rarity.label())
                                                            checked[is_boosted]
                                                            onchange={"toggleDriverBoost('" (driver_def.name) "', '" (driver_def.rarity.db_key()) "', this.checked, '" (driver_def.rarity.css_class()) "', '" (driver_def.rarity.label()) "')"};
                                                    }
                                                    td class=(driver_def.rarity.css_class()) { (driver_def.name) }
                                                    td { (driver_def.rarity.label()) }
                                                    td { (driver_def.series) }
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

            (PreEscaped(DRIVER_BOOST_JS))
        },
    )
}

const DRIVER_BOOST_JS: &str = r#"
<script>
function toggleDriverBoost(name, rarity, enabled, cssClass, label) {
    const container = document.getElementById('active-driver-boosts');
    const noBoosts = document.getElementById('no-driver-boosts');
    const key = name + ':' + rarity;
    const existing = container.querySelector(`[data-key="${key}"]`);

    if (enabled && !existing) {
        if (noBoosts) noBoosts.remove();
        const entry = document.createElement('div');
        entry.className = 'boost-entry';
        entry.dataset.key = key;
        entry.innerHTML = `
            <span class="boost-name ${cssClass || ''}">${name}</span>
            <span class="boost-cat">${label || rarity}</span>
            <input type="number" name="boost:${key}" min="1" max="100" step="1" value="10" class="compact">
            <span>%</span>
            <button type="button" class="btn-delete outline" onclick="toggleDriverBoost('${name}', '${rarity}', false)">×</button>
        `;
        container.appendChild(entry);
    } else if (!enabled && existing) {
        existing.remove();
        const cb = document.querySelector(`.driver-boost-check[data-name="${name}"][data-rarity="${rarity}"]`);
        if (cb) cb.checked = false;
        if (!container.querySelector('.boost-entry')) {
            const p = document.createElement('p');
            p.className = 'secondary';
            p.id = 'no-driver-boosts';
            p.textContent = 'No driver boosts active. Select drivers below.';
            container.appendChild(p);
        }
    }
}
</script>
"#;
