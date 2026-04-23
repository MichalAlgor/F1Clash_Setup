# Changelog

## [0.5.0] — 2026-04-23

### Added
- **Privacy-first analytics** — fire-and-forget page event capture with no third-party scripts; records path, device class, country (via swappable GeoIP provider), referrer, and response time; bot traffic filtered automatically
- **Analytics dashboard** (`/analytics`) — admin-only JSON API and UI showing unique visitors, top paths, referrers, countries, device breakdown, hourly/day-of-week distribution, engagement stats, and a feature funnel
- **Feature event instrumentation** — key actions (optimizer run, save, share, export/import, season switch, setup create/delete) are recorded as named feature events with JSONB properties
- **Upgrade Advisor** — shows which parts and drivers in your inventory are most valuable to upgrade next, ranked by the score gain per upgrade relative to your current optimizer result
- **Setup comparison** — select multiple setups from the list and compare their stats side-by-side with best/worst highlighting per stat
- **Setup edit** — edit route and pre-filled form for existing setups; previously only create and delete were supported
- **Shareable optimizer results** — optimizer results can be shared via a snapshot URL that captures part names, levels, and stats at share time; snapshot is immutable so links remain valid after inventory changes
- **Share setups** — Share button on the setup detail page creates a snapshot link just like the optimizer share
- **Share view count** — each visit to a shared setup increments a persistent `view_count` column (atomic `UPDATE...RETURNING`); displayed in the page subtitle with correct singular/plural
- **Share deduplication** — tapping Share on an unchanged setup returns the existing link instead of creating a new DB row; a SHA-256 hash over the sorted snapshot detects identical shares
- **Default/placeholder parts in setups** — setup slots can be left empty (shown as "Default · 1/1/1/1 · 1.00s pit") rather than requiring all 7 parts to be filled
- **Pit stop time in total score** — pit stop time now contributes to the total performance score used by the optimizer and setup view
- **Quick-start guide** — `/guide` page explains how to use the app for new users
- **Partial 2026 season data** — additional parts and drivers added to the 2026 catalog; coin upgrade costs corrected for Series 1–3

### Changed
- **Optimizer scoring** — `score_part_combo` now returns a 3-tuple `(min_priority, sum_priorities, total_performance)` so total performance is a clean tiebreaker that never pollutes priority comparison; for multiple priorities, the min-first bottleneck approach ensures no single stat is sacrificed
- **Custom optimizer result UI** — result page now uses the same `preset-card` layout as Presets: category/part merged into one cell with rarity colouring, `preset-figure` tables, score summary line, and matching save/share form style
- **Optimizer save form** — redesigned for better discoverability; editable name with inline pencil toggle, Save and Share buttons consistently placed
- **Share back link** — confirmation page after sharing links back to Setups or Optimizer depending on which flow created the share
- **Mobile UX** — responsive layout improvements across inventory, optimizer, and setup pages; stat columns collapse gracefully on small screens
- **drivers.json included in Docker image** — catalog seed file is now baked into the image so fresh deployments auto-populate without a separate step

### Fixed
- **Delete actions broken on Render** — Render's reverse proxy strips HTTP `DELETE` requests; all delete routes replaced with `POST /{id}/delete` and `hx-delete` updated to `hx-post` across inventory, drivers, setups, admin parts, and admin drivers
- **Pit stop score formula** — corrected multiplier from `200/7 ≈ 28.57` to `29.0`; updated all affected tests
- **Optimizer save/share with placeholder parts** — parts with `id = 0` (Default slots) are now skipped when writing hidden form inputs, preventing invalid IDs from being submitted
- **Analytics funnel** — `optimizer_presets` events now counted alongside `optimizer_run` in the "ran optimizer" funnel step
- CSS typo `display: flex-direction: column` → `display: flex; flex-direction: column` on the setup detail page (grid columns were not applying flex layout)

---

## [0.4.0] — 2026-04-17

### Added
- **Multi-user session isolation** — every visitor gets a private, independent workspace with no signup required; a UUID cookie (`user_session`) identifies each session, SHA-256 hashed before DB storage
- **Per-session active season** — each session independently tracks which season is active; switching seasons in one browser does not affect any other
- **Session ID display** — the Export / Import page shows the hashed session ID under a collapsed section, allowing data transfer to another browser by copying the cookie value
- **Export / Import combined page** — Export and Import are now a single page (`/import`) instead of two separate nav entries; includes session info, a download button, and the import form

### Changed
- All user-data queries (inventory, drivers, setups, boosts, optimizer) now filter by `session_id` — complete data isolation between visitors
- `user_session` cookie set with `HttpOnly; SameSite=Lax; Max-Age=31536000`; `SameSite=Lax` also covers CSRF for this app
- Optimizer `prune_category()` caps candidates at 10 per category to keep brute-force tractable with large inventories (8^6 = 262K combos vs unbounded)
- `drivers.json` committed to repo; removed 1700-line static `DRIVER_CATALOG` from `drivers_data.rs`

### Fixed
- FK constraint violation when bulk-saving or importing inventory — setup part/driver references are now NULLed before the inventory DELETE

---

## [0.3.0] — 2026-04-16

### Added
- **Season-scoped driver catalog** — drivers are now stored in PostgreSQL (`driver_catalog` + `driver_level_stats` tables), scoped per season, mirroring the parts catalog architecture
- **`drivers.json` seed file support** — place a `drivers.json` in the project root to auto-populate the driver catalog on startup (upserted, never deletes); if absent and the table is empty, the built-in static data is seeded for season "2025" automatically
- **Driver catalog admin UI** (`/admin/drivers`) — add, edit, and delete driver definitions directly from the browser; accessible via the Parts admin page
- **Driver catalog export** — download the full driver catalog (all seasons) as `drivers.json` from `/admin/drivers/export`
- **Multi-season driver support** — each season has its own independent set of drivers; new drivers can be added for future seasons without code changes

### Changed
- All routes and templates that previously looked up driver definitions from hardcoded static data (`drivers_data`) now use the DB-backed catalog; this affects drivers, setups, boosts, optimizer, and export/import
- Season selector now includes seasons defined in the driver catalog

---

## [0.2.5] — 2026-04-16

### Added
- **Battery part category** — new part type for the 2026 season with Overtake Mode secondary stat (Impact, Duration, Recharge Rate sub-stats)
- **Generic additional stat system** — replaces the hardcoded DRS column; any part category can declare its own secondary stat without schema changes; DRS data migrated automatically
- **Season-aware part categories** — each season declares which part slots are active (e.g. 2025 has Rear Wing; 2026 has Battery); optimizer, setup builder, and inventory adapt automatically
- **Season Settings admin page** (`/admin/seasons`) — configure which categories are active per season; create new seasons; season creation removed from the public season switcher
- **Season selector in nav** — inline dropdown replaces the separate season page; switching seasons reloads the current page
- **Card upgrade calculator** — enter how many cards you own for a part or driver; shows the highest reachable level and coin cost to get there, updated reactively via htmx without a page reload
- **Optimizer Presets tab** — runs 6 pre-defined optimizations (Speed, Cornering, Power Unit — each paired with Qualifying) and shows all results at once in a 3-group, 2-column layout; parts-only, no drivers
- **Criterion benchmarks** — `benches/optimizer.rs` provides a regression baseline for the brute-force optimizer using a realistic series-12 dataset

### Changed
- **Optimizer performance** — separated part and driver scoring; part combo and driver pair are now found independently, reducing evaluations from O(combos × pairs) to O(combos) + O(pairs); ~79× fewer iterations on a full series-12 inventory
- **Optimizer Custom tab** — existing custom optimizer moved to `/optimizer/custom`; `/optimizer` now shows the Presets tab by default
- **Inventory table** — more compact layout matching the driver inventory style; Series column removed; Cards and upgrade info merged into one reactive column
- **Session-based admin auth** — replaced HTTP Basic Auth with a password form embedded in the nav header; login sets a session cookie; admin link hidden when not logged in
- **Docker** — `rust:latest` base image; `static/` directory handled gracefully when empty; `docker-compose.yml` picks up `ADMIN_PASSWORD` from `.env` via `env_file`
- **Nav header** — sticky, slimmer height; season switcher is an inline dropdown; Admin link shown only when logged in (or when auth is disabled)

### Fixed
- Optimizer series limit inputs default to 12 so an empty field no longer causes a parse error
- `parts.json` category values deserialize correctly from snake_case (`"brakes"`, `"rear_wing"`, etc.)

---

## [0.2.0] — 2026-04-15

### Added
- **Season-scoped parts catalog** — catalog is now stored in PostgreSQL, scoped per season, instead of being hardcoded in Rust
- **`parts.json` seed file** — committed to the repo so fresh deployments auto-populate the catalog on first run (upserted on startup, never deletes)
- **Admin UI** (`/admin/parts`) — add, edit, and delete parts directly from the browser without touching code or redeploying; shows parts for the active season
- **Catalog export** — download the full catalog (all seasons) as `parts.json` from the admin UI to keep the seed file in sync
- **HTTP Basic Auth for admin routes** — set `ADMIN_PASSWORD` env var to protect `/admin/*`; if unset, admin is open (useful in local dev)
- **Multi-season catalog support** — each season has its own independent set of parts; switching seasons updates the catalog view across the whole app

---

## [0.1.0] — 2026-04-14

### Features
- **Parts inventory** — track which parts you own and at what upgrade level
- **Car setups** — build and compare setups from your owned parts, with full stat breakdowns
- **Optimizer** — brute-force finds the best part combination from your inventory given stat priorities and series limits
- **Drivers** — manage your driver inventory and include drivers in setups and the optimizer
- **Boosts** — apply percentage boosts to parts and drivers, reflected in setup stats and optimizer results
- **Seasons** — switch between seasons; inventory, setups, boosts, and drivers are all season-scoped
- **Export / Import** — export your inventory to JSON and import it back (useful for backups or moving between devices)
- **Rarity system** — parts are colour-coded by rarity (Common, Rare, Epic)
- **Series filtering** — optimizer can be limited to parts and drivers up to a given series number
