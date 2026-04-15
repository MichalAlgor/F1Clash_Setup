# Changelog

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
