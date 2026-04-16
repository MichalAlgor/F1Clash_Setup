# Changelog

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
