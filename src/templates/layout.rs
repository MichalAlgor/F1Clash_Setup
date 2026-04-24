use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::auth::AuthStatus;

pub fn page(title: &str, auth: &AuthStatus, content: Markup) -> Markup {
    page_inner(title, None, auth, content)
}

pub fn page_with_og(
    title: &str,
    og_title: &str,
    og_description: &str,
    auth: &AuthStatus,
    content: Markup,
) -> Markup {
    let extra = html! {
        meta property="og:title" content=(og_title);
        meta property="og:description" content=(og_description);
        meta property="og:type" content="website";
        meta property="og:site_name" content="F1 Clash Setup";
        meta name="twitter:card" content="summary";
        meta name="twitter:title" content=(og_title);
        meta name="twitter:description" content=(og_description);
    };
    page_inner(title, Some(extra), auth, content)
}

fn page_inner(
    title: &str,
    extra_head: Option<Markup>,
    auth: &AuthStatus,
    content: Markup,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "F1 Clash Setup — " (title) }
                @if let Some(ref extra) = extra_head { (extra) }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css";
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                style { (PreEscaped(CUSTOM_CSS)) }
            }
            body {
                header.container {
                    input #nav-toggle type="checkbox" {}
                    nav {
                        ul {
                            li {
                                a href="/" class="brand" {
                                    strong { "F1 Clash Setup" }
                                }
                            }
                            li.nav-toggle-li {
                                label.nav-toggle-label for="nav-toggle" { "☰" }
                            }
                        }
                        ul.nav-links {
                            li { a href="/inventory" { "Parts" } }
                            li { a href="/drivers" { "Drivers" } }
                            li { a href="/setups" { "Setups" } }
                            li { a href="/boosts" { "Boosts" } }
                            li { a href="/optimizer" { "Optimizer" } }
                            li { a href="/advisor" { "Advisor" } }
                            li { a href="/import" { "Export / Import" } }
                            @if !auth.enabled || auth.logged_in {
                                li { a href="/admin/parts" { "Admin" } }
                            }
                            li {
                                span hx-get="/api/season-selector" hx-trigger="load" hx-swap="outerHTML" {
                                    span class="season-badge" { "..." }
                                }
                            }
                            @if auth.enabled {
                                li class="auth-nav" {
                                    @if auth.logged_in {
                                        div class="auth-status" {
                                            span class="logged-in-text" { "Logged in" }
                                            form action="/api/logout" method="post" class="auth-form" {
                                                button type="submit" class="secondary outline" { "Logout" }
                                            }
                                        }
                                    } @else {
                                        form action="/api/login" method="post" class="auth-form" {
                                            input type="password" name="password" placeholder="Admin password" required class="auth-input";
                                            button type="submit" { "Login" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                main.container {
                    (content)
                }
                footer.container {
                    hr;
                    p style="text-align:center" {
                        small.secondary {
                            "F1 Clash Setup v" (env!("CARGO_PKG_VERSION"))
                            " · Your data is saved in this browser. "
                            a href="/import" { "Export" }
                            " to back it up."
                            " · "
                            a href="/guide" { "Quick-start guide" }
                            " · Made with ❤️ by "
                            a href="https://github.com/MichalAlgor/F1Clash_Setup" { "Mikele" }
                        }
                    }
                }
            }
        }
    }
}

const CUSTOM_CSS: &str = r#"
/* Use fluid container — less wasted side margin */
main.container, header.container, footer.container {
    max-width: 1400px;
}

.brand {
    text-decoration: none;
    font-size: 1.05rem;
    letter-spacing: -0.02em;
}

header.container {
    position: sticky;
    top: 0;
    z-index: 100;
    background: var(--pico-background-color);
    border-bottom: 1px solid var(--pico-muted-border-color);
    padding-top: 0 !important;
    padding-bottom: 0 !important;
}
header.container nav {
    padding: 0.2rem 0;
}
header.container nav ul {
    margin-bottom: 0;
}
header.container nav ul li a,
header.container nav ul li span {
    padding-top: 0.2rem;
    padding-bottom: 0.2rem;
}

/* Tighter page headers */
hgroup {
    margin-bottom: 1rem;
}
hgroup h1 {
    margin-bottom: 0.15rem;
}

/* Category sections */
h2 {
    margin-top: 1.25rem;
    margin-bottom: 0.5rem;
    padding-bottom: 0.3rem;
    border-bottom: 1px solid var(--pico-muted-border-color);
    font-size: 1rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

/* Compact tables */
figure {
    margin: 0.5rem 0;
}
table {
    font-size: 0.8rem;
    width: auto;
}
table th {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--pico-muted-color);
    font-weight: 600;
}
table td, table th {
    padding: 0.25rem 0.35rem;
}
table tfoot td {
    border-top: 2px solid var(--pico-muted-border-color);
    font-weight: 600;
}

/* Inline selects — fix overlap with chevron */
select.inline-select {
    margin: 0;
    padding: 2px 36px 2px 8px;
    width: auto;
    min-width: 64px;
    font-size: 0.8rem;
    display: inline-block;
}

/* Two-column category grid on wide screens */
.category-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 0 1.5rem;
}
@media (min-width: 992px) {
    .category-grid {
        grid-template-columns: 1fr 1fr;
    }
}
.category-grid > section {
    break-inside: avoid;
}

/* Bulk page table columns */
table.bulk-table th:first-child,
table.bulk-table td:first-child {
    width: 40%;
}
table.bulk-table th:nth-child(2),
table.bulk-table td:nth-child(2) {
    width: 20%;
    text-align: center;
}
table.bulk-table th:nth-child(3),
table.bulk-table td:nth-child(3) {
    width: 40%;
    text-align: right;
}

/* Number inputs in forms */
input[type="number"].compact {
    margin: 0;
    padding: 2px 8px;
    width: 72px;
    font-size: 0.85rem;
    text-align: center;
}

/* Compact action buttons inside table cells */
table td button {
    margin: 0;
    padding: 0.2rem 0.55rem;
    font-size: 0.8rem;
    line-height: 1;
}
/* Legacy class kept for compatibility */
button.btn-delete {
    margin: 0;
    padding: 0.2rem 0.55rem;
    font-size: 0.8rem;
    line-height: 1;
    cursor: pointer;
}

/* Tabs */
.boost-tabs {
    display: flex;
    gap: 0;
    margin-bottom: 1rem;
    border-bottom: 2px solid var(--pico-muted-border-color);
}
.boost-tab {
    padding: 0.5rem 1.25rem;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    cursor: pointer;
    font-size: 0.9rem;
    color: var(--pico-muted-color);
}
.boost-tab.active {
    color: var(--pico-color);
    border-bottom-color: var(--pico-primary);
    font-weight: 600;
}
.tab-content { display: none; }
.tab-content.active { display: block; }

/* Boost entries */
#active-boosts {
    margin-bottom: 1rem;
    padding: 0.75rem;
    border: 1px solid var(--pico-muted-border-color);
    border-radius: var(--pico-border-radius);
    min-height: 2.5rem;
}
.boost-entry {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.35rem 0;
}
.boost-entry + .boost-entry {
    border-top: 1px solid var(--pico-muted-border-color);
}
.boost-name {
    font-weight: 600;
    min-width: 120px;
}
.boost-cat {
    font-size: 0.75rem;
    color: var(--pico-muted-color);
    min-width: 80px;
}

/* Season selector in nav */
select.season-select {
    margin: 0;
    padding: 0.15rem 2rem 0.15rem 0.4rem;
    width: auto;
    min-width: 70px;
    font-size: 0.75rem;
    border-color: var(--pico-muted-border-color);
    background-color: transparent;
    display: inline-block;
}

/* Rarity colors */
.rarity-common { color: #4a90d9; }
.rarity-rare { color: #ed7d31; }
.rarity-epic { color: #b46dd8; }
.rarity-legendary { color: #ffd700; }
.rarity-prospect-std { color: #2ecc71; }
.rarity-prospect-turbo { color: #1abc9c; }
.rarity-podium { color: #e74c3c; }
.rarity-podium-legends { color: #ff6b6b; }

/* Cards input + upgrade tag */
.cards-cell {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    white-space: nowrap;
}
input.cards-input {
    width: 52px;
    height: auto;
    padding: 0.15rem 0.3rem;
    margin: 0;
    line-height: 1.2;
    font-size: 0.8rem;
    text-align: center;
    background: transparent;
    color: inherit;
    border: 1px solid var(--pico-muted-border-color);
    border-radius: var(--pico-border-radius);
    -moz-appearance: textfield;
}
input.cards-input::-webkit-outer-spin-button,
input.cards-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
}
.upgrade-tag {
    font-size: 0.75rem;
    white-space: nowrap;
}
.upgrade-tag.secondary {
    color: var(--pico-muted-color);
}

/* Optimizer tabs */
.optimizer-tabs {
    display: flex;
    gap: 0;
    border-bottom: 2px solid var(--pico-muted-border-color);
    margin-bottom: 1rem;
}
.optimizer-tab {
    padding: 0.4rem 1.2rem;
    text-decoration: none;
    color: var(--pico-muted-color);
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
}
.optimizer-tab.active {
    color: var(--pico-color);
    border-bottom-color: var(--pico-primary);
    font-weight: 600;
}

/* Preset result cards */
.preset-group {
    margin-bottom: 1.5rem;
}
.preset-pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
}
@media (max-width: 900px) {
    .preset-pair { grid-template-columns: 1fr; }
}
.preset-card {
    border: 1px solid var(--pico-muted-border-color);
    border-radius: var(--pico-border-radius);
    padding: 0.75rem;
}
.preset-card table {
    font-size: 0.75rem;
}

/* Auth nav */
.auth-nav {
    display: flex;
    align-items: center;
}
.auth-form {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0;
}
.auth-form input.auth-input {
    margin: 0;
    padding: 0.2rem 0.5rem;
    width: 140px;
    font-size: 0.8rem;
    height: auto;
}
.auth-form button {
    margin: 0;
    padding: 0.2rem 0.6rem;
    font-size: 0.8rem;
}
.auth-status {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}
.logged-in-text {
    font-size: 0.8rem;
    color: var(--pico-muted-color);
}

/* Footer */
footer.container {
    padding-bottom: 1rem;
}
footer hr {
    margin-bottom: 0.5rem;
}
footer p {
    margin: 0;
}

/* ═══════════════════════════════════════════════════════════
   MOBILE RESPONSIVE STYLES
   ═══════════════════════════════════════════════════════════ */

/* --- Mobile navigation (hamburger) --- */
#nav-toggle {
    display: none;
}
.nav-toggle-li {
    display: none !important;
}

@media (max-width: 768px) {
    .nav-toggle-li {
        display: list-item !important;
    }
    .nav-toggle-label {
        cursor: pointer;
        font-size: 1.5rem;
        padding: 0.3rem 0.5rem;
        line-height: 1;
    }
    .nav-links {
        display: none !important;
        flex-direction: column;
        width: 100%;
        padding: 0.5rem 0;
    }
    #nav-toggle:checked ~ nav .nav-links {
        display: flex !important;
    }
    header.container nav {
        flex-wrap: wrap;
    }
    header.container nav ul li {
        width: 100%;
    }
    header.container nav ul li a,
    header.container nav ul li span {
        min-height: 44px;
        display: flex;
        align-items: center;
    }
    /* Auth form stacks vertically on mobile */
    .auth-form {
        flex-direction: column;
        align-items: stretch;
    }
    .auth-form input.auth-input {
        width: 100%;
    }
}

/* --- Responsive table → card layout on mobile --- */
@media (max-width: 768px) {
    /* PicoCSS overrides for card mode */
    figure:has(.responsive-table) {
        margin: 0;
        overflow: visible;
    }
    .responsive-table {
        width: 100% !important;
        border-collapse: separate;
        border-spacing: 0;
    }
    .responsive-table thead {
        display: none;
    }

    /* Card container */
    .responsive-table tbody tr {
        display: flex;
        flex-wrap: wrap;
        border: 1px solid var(--pico-muted-border-color);
        border-radius: var(--pico-border-radius);
        padding: 0.5rem;
        margin-bottom: 0.75rem;
        background: var(--pico-card-background-color);
    }

    /* Default: metadata cells — full-width label-value rows */
    .responsive-table tbody td {
        width: 100%;
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.15rem 0;
        border: none;
    }
    .responsive-table tbody td::before {
        content: attr(data-label);
        font-weight: 600;
        font-size: 0.75rem;
        text-transform: uppercase;
        color: var(--pico-muted-color);
        min-width: 60px;
        flex-shrink: 0;
    }

    /* First cell = card header */
    .responsive-table tbody td:first-child {
        font-weight: 600;
        font-size: 1rem;
        border-bottom: 1px solid var(--pico-muted-border-color);
        margin-bottom: 0.25rem;
        padding-bottom: 0.25rem;
    }
    .responsive-table tbody td:first-child::before {
        content: none;
    }

    /* Stat cells — compact grid: label on top, value below */
    .responsive-table tbody td.stat-cell {
        width: auto;
        flex: 1 1 48px;
        flex-direction: column;
        align-items: center;
        justify-content: flex-start;
        text-align: center;
        padding: 0.3rem 0.1rem;
        order: 1;
        min-width: 0;
    }
    .responsive-table tbody td.stat-cell::before {
        min-width: auto;
        text-align: center;
        margin-bottom: 0.1rem;
        font-size: 0.65rem;
    }

    /* Action cell (delete) — right-aligned at bottom */
    .responsive-table tbody td.action-cell {
        order: 2;
        justify-content: flex-end;
        padding-top: 0.25rem;
    }
    .responsive-table tbody td.action-cell::before {
        content: none;
    }

    /* tfoot (totals row) */
    .responsive-table tfoot tr {
        display: flex;
        flex-wrap: wrap;
        border: 2px solid var(--pico-primary);
        border-radius: var(--pico-border-radius);
        padding: 0.5rem;
        margin-top: 0.5rem;
    }
    .responsive-table tfoot td {
        display: flex;
        justify-content: space-between;
        padding: 0.1rem 0;
        border: none;
        width: 100%;
    }
    .responsive-table tfoot td::before {
        content: none;
    }
    .responsive-table tfoot td:first-child {
        font-weight: 600;
        font-size: 0.9rem;
        border-bottom: 1px solid var(--pico-muted-border-color);
        margin-bottom: 0.25rem;
        padding-bottom: 0.25rem;
    }
    .responsive-table tfoot td.stat-cell {
        width: auto;
        flex: 1 1 48px;
        flex-direction: column;
        align-items: center;
        text-align: center;
        padding: 0.3rem 0.1rem;
        min-width: 0;
    }
    .responsive-table tfoot td.stat-cell::before {
        content: attr(data-label);
        font-weight: 600;
        font-size: 0.65rem;
        text-transform: uppercase;
        color: var(--pico-muted-color);
        margin-bottom: 0.1rem;
    }
}

/* --- Optimizer: migrated inline styles to classes --- */
.series-limits-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
}
.back-link {
    margin-bottom: 1rem;
    display: inline-block;
}
.preset-figure {
    margin: 0 0 0.5rem;
}
.preset-score {
    margin: 0.25rem 0;
}
.preset-save-form,
.custom-save-form {
    margin-top: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
}
/* Hidden checkbox toggle */
.save-toggle {
    display: none !important;
}
/* Name row: visible text + pencil icon */
.save-name-row {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.35rem;
    min-width: 0;
}
.save-name-display {
    font-size: 0.8rem;
    color: var(--pico-muted-color);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.save-edit-btn {
    cursor: pointer;
    font-size: 0.75rem;
    opacity: 0.5;
    flex-shrink: 0;
    line-height: 1;
}
.save-edit-btn:hover {
    opacity: 1;
}
/* Editable input — hidden by default */
.save-name-edit {
    display: none !important;
    flex: 1;
    margin: 0;
    padding: 0.2rem 0.5rem;
    font-size: 0.8rem;
    height: auto;
}
/* When toggle checked: swap display↔input */
.save-toggle:checked ~ .save-name-row {
    display: none;
}
.save-toggle:checked ~ .save-name-edit {
    display: block !important;
}
.save-form-btn {
    white-space: nowrap;
    width: auto !important;
    margin: 0;
    padding: 0.25rem 0.75rem;
    font-size: 0.8rem;
}
.preset-form-btns {
    display: flex;
    gap: 0.35rem;
    flex-shrink: 0;
}

@media (max-width: 768px) {
    .series-limits-grid {
        grid-template-columns: 1fr;
    }
    /* Save form: stack vertically on mobile */
    .preset-save-form,
    .custom-save-form {
        flex-direction: column;
        align-items: stretch;
    }
    .save-name-row {
        justify-content: space-between;
    }
    .save-name-edit {
        width: 100%;
    }
    /* Preset cards: hide individual stats, show name + total only */
    .preset-parts-table .stat-cell,
    .preset-parts-table .stat-header {
        display: none !important;
    }
    .preset-parts-table tfoot .stat-cell {
        display: none !important;
    }
}

/* --- Touch targets & form adjustments --- */
@media (max-width: 768px) {
    /* Minimum touch target for interactive elements */
    button,
    [role="button"],
    a[role="button"] {
        min-height: 44px;
    }
    /* Submit buttons go full-width on mobile */
    form > button[type="submit"],
    form > input[type="submit"] {
        width: 100%;
    }
    /* Select dropdowns — smaller text for long option labels */
    select {
        font-size: 0.85rem;
    }
    /* Compact number inputs (boosts) — ensure tappable */
    input[type="number"].compact {
        min-height: 36px;
        width: 70px;
    }
    /* Checkboxes — larger on mobile */
    input[type="checkbox"] {
        width: 20px;
        height: 20px;
    }
    /* Fieldset labels — ensure tappable */
    fieldset label {
        padding: 0.3rem 0;
        min-height: 44px;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    /* Inline-select dropdowns (bulk edit, level selectors) */
    select.inline-select {
        min-height: 36px;
        font-size: 0.85rem;
    }
    /* Boost tabs */
    .boost-tab {
        padding: 0.75rem 1rem;
        min-height: 44px;
    }
}

/* --- Share page (confirmation screen) --- */
.share-url-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
    margin: 1rem 0;
}
.share-url {
    flex: 1;
    padding: 0.4rem 0.75rem;
    border: 1px solid var(--pico-muted-border-color);
    border-radius: var(--pico-border-radius);
    font-size: 0.85rem;
    word-break: break-all;
}

/* --- Share view page redesign --- */
.share-hero {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    padding: 1.25rem 1.5rem;
    margin-bottom: 1.5rem;
    background: var(--pico-card-background-color);
    border: 1px solid var(--pico-muted-border-color);
    border-left: 3px solid var(--pico-primary);
    border-radius: var(--pico-border-radius);
}
.share-hero-left { flex: 1; min-width: 0; }
.share-label {
    font-size: 0.6rem;
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--pico-muted-color);
    margin: 0 0 0.35rem;
}
.share-hero h1 {
    font-size: 1.6rem;
    margin: 0 0 0.25rem;
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
}
.share-hero-sub {
    font-size: 0.8rem;
    color: var(--pico-muted-color);
    margin: 0;
}
.share-season-badge {
    font-size: 0.7rem;
    font-weight: 600;
    padding: 0.15rem 0.6rem;
    border-radius: 999px;
    background: rgba(46, 204, 113, 0.12);
    color: #2ecc71;
    border: 1px solid rgba(46, 204, 113, 0.35);
    letter-spacing: 0.03em;
    white-space: nowrap;
}
.share-hero-right {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.5rem;
    flex-shrink: 0;
}
.share-views-count {
    font-size: 0.78rem;
    color: var(--pico-muted-color);
}
.share-total-badge {
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.07em;
    text-transform: uppercase;
    padding: 0.3rem 0.8rem;
    background: var(--pico-primary);
    color: var(--pico-primary-inverse);
    border-radius: var(--pico-border-radius);
    white-space: nowrap;
}
.share-layout {
    display: grid;
    grid-template-columns: 1fr;
    gap: 1.5rem;
    align-items: start;
}
@media (min-width: 900px) {
    .share-layout { grid-template-columns: 1fr 260px; }
}
h2.share-section-h2 {
    font-size: 0.68rem;
    color: var(--pico-muted-color);
    margin: 1.25rem 0 0.5rem;
    padding: 0.2rem 0 0.2rem 0.6rem;
    border-bottom: none;
    border-left: 3px solid var(--pico-primary);
}
h2.share-section-h2:first-child { margin-top: 0; }
.share-total-cell { color: #4fc3f7 !important; font-weight: 700; }
.share-score-bar {
    display: flex;
    align-items: center;
    gap: 2rem;
    padding: 0.75rem 1rem;
    margin-top: 1rem;
    background: var(--pico-card-background-color);
    border: 1px solid var(--pico-muted-border-color);
    border-radius: var(--pico-border-radius);
    flex-wrap: wrap;
}
.share-score-combined {
    font-size: 0.95rem;
    font-weight: 700;
    color: #ffd700;
}
.share-score-stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 70px;
}
.share-score-stat-value {
    font-size: 1.05rem;
    font-weight: 700;
}
.share-score-stat-label {
    font-size: 0.6rem;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--pico-muted-color);
}
.share-sidebar-card {
    background: var(--pico-card-background-color);
    border: 1px solid var(--pico-muted-border-color);
    border-radius: var(--pico-border-radius);
    padding: 1rem;
    position: sticky;
    top: 4rem;
}
.share-sidebar-card h3 {
    font-size: 0.62rem;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--pico-muted-color);
    margin: 0 0 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid var(--pico-muted-border-color);
}
.share-summary-dl {
    margin: 0;
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.3rem 0.75rem;
    align-items: baseline;
}
.share-summary-dl dt {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--pico-muted-color);
    white-space: nowrap;
}
.share-summary-dl dd {
    font-size: 0.85rem;
    font-weight: 500;
    margin: 0;
    color: var(--pico-color);
    word-break: break-word;
}
.share-compare-section { margin-top: 2rem; }
@media (max-width: 768px) {
    .share-hero { flex-direction: column; }
    .share-hero-right { align-items: flex-start; flex-direction: row; flex-wrap: wrap; }
    .share-score-bar { gap: 1rem; }
}

/* --- Setup Comparison --- */
.setups-actions {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    margin-bottom: 1rem;
    flex-wrap: wrap;
}
.compare-col {
    width: 1rem;
    padding: 0.25rem !important;
}
.compare-table {
    width: 100%;
    font-size: 0.85rem;
}
.compare-table th:first-child,
.compare-table td:first-child {
    text-align: left;
    white-space: nowrap;
    padding-right: 1rem;
}
.compare-table td {
    text-align: center;
}
.compare-best {
    color: var(--pico-ins-color, #2ecc71);
    font-weight: 700;
}
.compare-worst {
    color: var(--pico-del-color, #e74c3c);
}

/* --- Upgrade Advisor --- */
.upgrade-positive {
    color: var(--pico-ins-color, #2ecc71);
}
.ready-badge {
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    background: var(--pico-ins-color, #2ecc71);
    color: #000;
    padding: 0.1rem 0.3rem;
    border-radius: 0.25rem;
    letter-spacing: 0.04em;
    vertical-align: middle;
}
.stat-delta {
    font-size: 0.75rem;
    color: var(--pico-muted-color);
}
.stat-delta span {
    white-space: nowrap;
}

/* --- Category icon badge (matches in-game red pill style) --- */
.cat-icon {
    width: 20px;
    height: 20px;
    padding: 3px;
    background: #b71c1c;
    border-radius: 5px;
    vertical-align: middle;
    margin-right: 0.35rem;
    display: inline-block;
    flex-shrink: 0;
}
h2:has(.cat-icon) {
    display: flex;
    align-items: center;
}

/* --- Admin action bar --- */
.admin-actions {
    display: flex;
    gap: 1rem;
    flex-wrap: wrap;
    margin-bottom: 1rem;
}

/* --- Boosts & admin adjustments on narrow screens --- */
@media (max-width: 480px) {
    .boost-entry {
        flex-wrap: wrap;
    }
    .boost-name {
        min-width: 100%;
    }
}
"#;
