use maud::{html, Markup, PreEscaped, DOCTYPE};

use crate::auth::AuthStatus;

pub fn page(title: &str, auth: &AuthStatus, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme="dark" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "F1 Clash Setup — " (title) }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css";
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                style { (PreEscaped(CUSTOM_CSS)) }
            }
            body {
                header.container {
                    nav {
                        ul {
                            li {
                                a href="/" class="brand" {
                                    strong { "F1 Clash Setup" }
                                }
                            }
                        }
                        ul {
                            li { a href="/inventory" { "Parts" } }
                            li { a href="/drivers" { "Drivers" } }
                            li { a href="/setups" { "Setups" } }
                            li { a href="/boosts" { "Boosts" } }
                            li { a href="/optimizer" { "Optimizer" } }
                            li { a href="/export" { "Export" } }
                            li { a href="/import" { "Import" } }
                            @if auth.logged_in {
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
    font-size: 1.15rem;
    letter-spacing: -0.02em;
}

header.container nav {
    padding: 0.5rem 0;
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

/* Delete button */
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
"#;
