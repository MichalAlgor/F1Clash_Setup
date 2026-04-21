use maud::{Markup, html};

use super::layout;
use crate::auth::AuthStatus;

pub fn guide_page(auth: &AuthStatus) -> Markup {
    layout::page(
        "Quick-Start Guide",
        auth,
        html! {
            hgroup {
                h1 { "Getting Started" }
                p { "Find the best car setup from the parts you actually own — without spreadsheets." }
            }

            p {
                "F1 Clash Setup Manager analyses every part combination from your inventory and tells you the optimal setup for any race focus. Follow these three steps to get going."
            }

            ol {
                li {
                    article {
                        hgroup {
                            h2 { "Step 1 — Add your parts" }
                            p { "Record which parts you own and at what level." }
                        }
                        p {
                            "Open the "
                            strong { "Bulk Parts" }
                            " form. For each part you own, select its current upgrade level. Parts left at the default are treated as not owned and won't appear in optimizer results."
                        }
                        a href="/inventory/bulk" role="button" { "Open Bulk Parts form →" }
                    }
                }
                li {
                    article {
                        hgroup {
                            h2 { "Step 2 — Run the Optimizer" }
                            p { "Pick a race focus and find the best setup combination." }
                        }
                        p {
                            "Go to the "
                            strong { "Optimizer" }
                            " and choose a stat focus: Speed, Cornering, or Power Unit (each paired with Qualifying). The optimizer tests every valid combination from your inventory and returns the best results. You can also filter by series if you're in a lower-tier race."
                        }
                        a href="/optimizer" role="button" class="outline" { "Open Optimizer →" }
                    }
                }
                li {
                    article {
                        hgroup {
                            h2 { "Step 3 — Save and compare" }
                            p { "Name your setup and compare different configurations side-by-side." }
                        }
                        p {
                            "From any optimizer result you can save the setup with a name. Visit "
                            strong { "Setups" }
                            " to see all your saved configurations and compare them side-by-side across every stat column — useful when choosing between a Speed build and a Cornering build for a specific track."
                        }
                        a href="/setups" role="button" class="outline" { "View Setups →" }
                    }
                }
            }

            p.secondary style="font-size:0.85rem;margin-top:1.5rem" {
                "Your data is stored in this browser session. Use "
                a href="/import" { "Export / Import" }
                " to back it up or move it to another device."
            }
        },
    )
}
