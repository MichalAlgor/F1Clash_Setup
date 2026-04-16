use criterion::{black_box, criterion_group, criterion_main, Criterion};

use f1clash_setup::data::StatPriorities;
use f1clash_setup::models::driver::{DriverInventoryItem, DriverStats};
use f1clash_setup::models::part::{PartCategory, Stats};
use f1clash_setup::models::setup::InventoryItem;
use f1clash_setup::optimizer_core::{
    DriverPriorities, ResolvedDriver, ResolvedPart, run_brute_force,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_part(id: i32, speed: i32, cornering: i32, power_unit: i32, qualifying: i32) -> ResolvedPart {
    ResolvedPart {
        item: InventoryItem {
            id,
            part_name: format!("Part {id}"),
            level: 8,
            cards_owned: 0,
        },
        stats: Stats {
            speed,
            cornering,
            power_unit,
            qualifying,
            pit_stop_time: 0.30,
            additional_stat_value: 0,
        },
        rarity_css_class: "rarity-epic",
    }
}

fn make_driver(id: i32, total: i32) -> ResolvedDriver {
    let per = total / 5;
    ResolvedDriver {
        item: DriverInventoryItem {
            id,
            driver_name: format!("Driver {id}"),
            rarity: "Epic".to_string(),
            level: 3,
            cards_owned: 0,
        },
        stats: DriverStats {
            overtaking: per,
            defending: per,
            qualifying: per,
            race_start: per,
            tyre_management: total - per * 4,
        },
    }
}

fn build_driver_pairs(
    drivers: &[ResolvedDriver],
) -> Vec<(Option<usize>, Option<usize>)> {
    let mut pairs = vec![(None, None)];
    for i in 0..drivers.len() {
        pairs.push((Some(i), None));
        for j in (i + 1)..drivers.len() {
            pairs.push((Some(i), Some(j)));
        }
    }
    pairs
}

// ── Benchmark dataset ─────────────────────────────────────────────────────────
//
// Mirrors a realistic series-12 inventory:
//   - 6 categories × 4 parts each  →  4^6 = 4,096 part combos
//   - 6 drivers  →  1 + 6 + 15 = 22 driver pairs
//   - Total evaluations: 4,096 × 22 ≈ 90,000

fn build_series12_data() -> (
    Vec<Vec<ResolvedPart>>,
    Vec<PartCategory>,
    Vec<ResolvedDriver>,
) {
    let categories = vec![
        PartCategory::Engine,
        PartCategory::FrontWing,
        PartCategory::RearWing,
        PartCategory::Suspension,
        PartCategory::Brakes,
        PartCategory::Gearbox,
    ];

    // 4 parts per category with varied stats typical of series-12 epics
    let parts_per_cat: Vec<Vec<ResolvedPart>> = vec![
        // Engine
        vec![
            make_part(1,  48, 19, 19, 21),
            make_part(2,  50, 20, 20, 23),
            make_part(3,  16, 15, 42, 17),
            make_part(4,  18, 17, 47, 19),
        ],
        // FrontWing
        vec![
            make_part(10, 46, 20, 19, 17),
            make_part(11, 48, 21, 20, 18),
            make_part(12, 17, 42, 15, 16),
            make_part(13, 19, 45, 16, 17),
        ],
        // RearWing
        vec![
            make_part(20, 42, 16, 15, 17),
            make_part(21, 45, 17, 16, 18),
            make_part(22, 11, 46, 12, 13),
            make_part(23, 12, 48, 13, 14),
        ],
        // Suspension
        vec![
            make_part(30, 49, 17, 18, 20),
            make_part(31, 51, 18, 19, 21),
            make_part(32, 21, 17, 48, 21),
            make_part(33, 21, 46, 19, 17),
        ],
        // Brakes
        vec![
            make_part(40, 19, 17, 49, 17),
            make_part(41, 20, 18, 51, 18),
            make_part(42, 16, 42, 17, 14),
            make_part(43, 17, 45, 18, 15),
        ],
        // Gearbox
        vec![
            make_part(50, 42, 15, 17, 16),
            make_part(51, 44, 16, 18, 17),
            make_part(52, 17, 20, 46, 19),
            make_part(53, 19, 22, 50, 21),
        ],
    ];

    let drivers = vec![
        make_driver(1, 350),
        make_driver(2, 330),
        make_driver(3, 325),
        make_driver(4, 325),
        make_driver(5, 315),
        make_driver(6, 280),
    ];

    (parts_per_cat, categories, drivers)
}

// ── Benchmarks ────────────────────────────────────────────────────────────────

fn bench_presets(c: &mut Criterion) {
    let (parts_per_cat, categories, drivers) = build_series12_data();
    let driver_pairs = build_driver_pairs(&drivers);
    let driver_priorities = DriverPriorities::default();

    let preset_priorities = [
        ("speed",           StatPriorities { speed: true,       ..Default::default() }),
        ("speed+qual",      StatPriorities { speed: true, qualifying: true, ..Default::default() }),
        ("cornering",       StatPriorities { cornering: true,   ..Default::default() }),
        ("cornering+qual",  StatPriorities { cornering: true, qualifying: true, ..Default::default() }),
        ("power_unit",      StatPriorities { power_unit: true,  ..Default::default() }),
        ("power_unit+qual", StatPriorities { power_unit: true, qualifying: true, ..Default::default() }),
    ];

    let mut group = c.benchmark_group("optimizer_presets");

    for (name, prio) in &preset_priorities {
        group.bench_function(*name, |b| {
            b.iter(|| {
                run_brute_force(
                    black_box(&parts_per_cat),
                    black_box(&categories),
                    black_box(&driver_pairs),
                    black_box(&drivers),
                    black_box(prio),
                    black_box(&driver_priorities),
                )
            });
        });
    }

    group.finish();
}

fn bench_all_presets_combined(c: &mut Criterion) {
    let (parts_per_cat, categories, drivers) = build_series12_data();
    let driver_pairs = build_driver_pairs(&drivers);
    let driver_priorities = DriverPriorities::default();

    let priorities: Vec<StatPriorities> = vec![
        StatPriorities { speed: true,                         ..Default::default() },
        StatPriorities { speed: true, qualifying: true,       ..Default::default() },
        StatPriorities { cornering: true,                     ..Default::default() },
        StatPriorities { cornering: true, qualifying: true,   ..Default::default() },
        StatPriorities { power_unit: true,                    ..Default::default() },
        StatPriorities { power_unit: true, qualifying: true,  ..Default::default() },
    ];

    c.bench_function("optimizer_all_6_presets", |b| {
        b.iter(|| {
            for prio in &priorities {
                run_brute_force(
                    black_box(&parts_per_cat),
                    black_box(&categories),
                    black_box(&driver_pairs),
                    black_box(&drivers),
                    black_box(prio),
                    black_box(&driver_priorities),
                );
            }
        });
    });
}

criterion_group!(benches, bench_presets, bench_all_presets_combined);
criterion_main!(benches);
