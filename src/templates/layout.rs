use maud::{html, Markup, DOCTYPE};

pub fn page(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "F1 Clash Setup — " (title) }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css";
                script src="https://unpkg.com/htmx.org@2.0.4" {}
            }
            body {
                nav.container {
                    ul {
                        li { a href="/" { strong { "F1 Clash Setup" } } }
                    }
                    ul {
                        li { a href="/parts" { "Parts" } }
                        li { a href="/setups" { "Setups" } }
                    }
                }
                main.container {
                    (content)
                }
            }
        }
    }
}
