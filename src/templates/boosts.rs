use maud::{Markup, PreEscaped, html};

use crate::auth::AuthStatus;
use crate::drivers_data::{DriverCategory, DriverRarity};
use crate::models::driver::{DriverBoost, OwnedDriverDefinition};
use crate::models::part::{OwnedPartDefinition, PartCategory};
use crate::models::setup::Boost;

pub fn page(
    part_boosts: &[Boost],
    driver_boosts: &[DriverBoost],
    catalog: &[OwnedPartDefinition],
    drivers_catalog: &[OwnedDriverDefinition],
    auth: &AuthStatus,
) -> Markup {
    let boosted_part_names: Vec<&str> = part_boosts.iter().map(|b| b.part_name.as_str()).collect();
    let boosted_driver_keys: Vec<(String, String)> = driver_boosts
        .iter()
        .map(|b| (b.driver_name.clone(), b.rarity.clone()))
        .collect();
    let has_any_boost = !part_boosts.is_empty() || !driver_boosts.is_empty();

    super::layout::page(
        "Boosts",
        auth,
        html! {
            hgroup {
                h1 { "Boosts" }
                p { "Manage part and driver boosts" }
            }

            form method="post" action="/boosts" id="boost-form" {
                // Unified active boosts panel
                div id="active-boosts" {
                    @if !has_any_boost {
                        p.secondary #no-boosts { "No boosts active. Select parts or drivers below." }
                    }
                    @for boost in part_boosts {
                        @if let Some(part_def) = catalog.iter().find(|p| p.name == boost.part_name) {
                            div class="boost-entry" data-part=(boost.part_name) data-type="part" {
                                span class={"boost-name " (part_def.rarity_css_class())} { (boost.part_name) }
                                span class="boost-cat" { (part_def.category.display_name()) }
                                input type="number"
                                    name={"part:" (boost.part_name)}
                                    min="1" max="100" step="1"
                                    value=(boost.percentage)
                                    class="compact";
                                span { "%" }
                                button type="button" class="btn-delete outline"
                                    onclick={"togglePartBoost('" (boost.part_name) "', false)"} { "×" }
                            }
                        }
                    }
                    @for boost in driver_boosts {
                        @let rarity_css = DriverRarity::from_db(&boost.rarity).map_or("", |r| r.css_class());
                        div class="boost-entry" data-key={"d:" (boost.driver_name) ":" (boost.rarity)} data-type="driver" {
                            span class={"boost-name " (rarity_css)} { (boost.driver_name) }
                            span class="boost-cat" { (boost.rarity) }
                            input type="number"
                                name={"driver:" (boost.driver_name) ":" (boost.rarity)}
                                min="1" max="100" step="1"
                                value=(boost.percentage)
                                class="compact";
                            span { "%" }
                            button type="button" class="btn-delete outline"
                                onclick={"toggleDriverBoost('" (boost.driver_name) "', '" (boost.rarity) "', false)"} { "×" }
                        }
                    }
                }

                button type="submit" { "Save All Boosts" }

                // Tab buttons for selection area
                div class="boost-tabs" {
                    button type="button" class="boost-tab active" data-tab="parts-tab" onclick="switchTab('parts-tab')" { "Parts" }
                    button type="button" class="boost-tab" data-tab="drivers-tab" onclick="switchTab('drivers-tab')" { "Drivers" }
                }

                // ===== PARTS SELECTION =====
                div id="parts-tab" class="tab-content active" {
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
                                                    th {}
                                                    th { "Part" }
                                                    th { "Series" }
                                                }
                                            }
                                            tbody {
                                                @for part_def in &parts {
                                                    @let is_boosted = boosted_part_names.contains(&part_def.name.as_str());
                                                    tr {
                                                        td {
                                                            input type="checkbox"
                                                                class="boost-check"
                                                                data-part=(part_def.name)
                                                                data-category=(category.display_name())
                                                                data-css=(part_def.rarity_css_class())
                                                                checked[is_boosted]
                                                                onchange={"togglePartBoost('" (part_def.name) "', this.checked, '" (category.display_name()) "', '" (part_def.rarity_css_class()) "')"};
                                                        }
                                                        td class=(part_def.rarity_css_class()) { (part_def.name) }
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

                // ===== DRIVERS SELECTION =====
                div id="drivers-tab" class="tab-content" {
                    div class="category-grid" {
                        @for category in DriverCategory::all() {
                            @let drivers: Vec<_> = drivers_catalog.iter()
                                .filter(|d| DriverRarity::from_db(&d.rarity)
                                    .is_some_and(|r| r.category() == *category))
                                .collect();
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
                                                    @let is_boosted = boosted_driver_keys.iter()
                                                        .any(|(n, r)| n == &driver_def.name && r == &driver_def.rarity);
                                                    @let d_rarity_css = DriverRarity::from_db(&driver_def.rarity).map_or("", |r| r.css_class());
                                                    tr {
                                                        td {
                                                            input type="checkbox"
                                                                class="driver-boost-check"
                                                                data-name=(driver_def.name)
                                                                data-rarity=(driver_def.rarity)
                                                                data-css=(d_rarity_css)
                                                                data-label=(driver_def.rarity)
                                                                checked[is_boosted]
                                                                onchange={"toggleDriverBoost('" (driver_def.name) "', '" (driver_def.rarity) "', this.checked, '" (d_rarity_css) "', '" (driver_def.rarity) "')"};
                                                        }
                                                        td class=(d_rarity_css) { (driver_def.name) }
                                                        td { (driver_def.rarity) }
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
            }

            (PreEscaped(BOOSTS_JS))
        },
    )
}

const BOOSTS_JS: &str = r#"
<script>
function switchTab(tabId) {
    document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
    document.querySelectorAll('.boost-tab').forEach(el => el.classList.remove('active'));
    document.getElementById(tabId).classList.add('active');
    document.querySelector(`.boost-tab[data-tab="${tabId}"]`).classList.add('active');
}

function hideNoBoosts() {
    const nb = document.getElementById('no-boosts');
    if (nb) nb.remove();
}

function showNoBoostsIfEmpty() {
    const container = document.getElementById('active-boosts');
    if (!container.querySelector('.boost-entry')) {
        const p = document.createElement('p');
        p.className = 'secondary';
        p.id = 'no-boosts';
        p.textContent = 'No boosts active. Select parts or drivers below.';
        container.appendChild(p);
    }
}

function togglePartBoost(partName, enabled, category, cssClass) {
    const container = document.getElementById('active-boosts');
    const existing = container.querySelector(`.boost-entry[data-part="${partName}"]`);

    if (enabled && !existing) {
        hideNoBoosts();
        const entry = document.createElement('div');
        entry.className = 'boost-entry';
        entry.dataset.part = partName;
        entry.dataset.type = 'part';
        entry.innerHTML = `
            <span class="boost-name ${cssClass || ''}">${partName}</span>
            <span class="boost-cat">${category || ''}</span>
            <input type="number" name="part:${partName}" min="1" max="100" step="1" value="10" class="compact">
            <span>%</span>
            <button type="button" class="btn-delete outline" onclick="togglePartBoost('${partName}', false)">×</button>
        `;
        container.appendChild(entry);
    } else if (!enabled && existing) {
        existing.remove();
        const cb = document.querySelector(`.boost-check[data-part="${partName}"]`);
        if (cb) cb.checked = false;
        showNoBoostsIfEmpty();
    }
}

function toggleDriverBoost(name, rarity, enabled, cssClass, label) {
    const container = document.getElementById('active-boosts');
    const key = 'd:' + name + ':' + rarity;
    const existing = container.querySelector(`.boost-entry[data-key="${key}"]`);

    if (enabled && !existing) {
        hideNoBoosts();
        const entry = document.createElement('div');
        entry.className = 'boost-entry';
        entry.dataset.key = key;
        entry.dataset.type = 'driver';
        entry.innerHTML = `
            <span class="boost-name ${cssClass || ''}">${name}</span>
            <span class="boost-cat">${label || rarity}</span>
            <input type="number" name="driver:${name}:${rarity}" min="1" max="100" step="1" value="10" class="compact">
            <span>%</span>
            <button type="button" class="btn-delete outline" onclick="toggleDriverBoost('${name}', '${rarity}', false)">×</button>
        `;
        container.appendChild(entry);
    } else if (!enabled && existing) {
        existing.remove();
        const cb = document.querySelector(`.driver-boost-check[data-name="${name}"][data-rarity="${rarity}"]`);
        if (cb) cb.checked = false;
        showNoBoostsIfEmpty();
    }
}
</script>
"#;
