# Details

Date : 2026-02-06 15:54:57

Directory c:\\pathfinder

Total : 46 files,  63923 codes, 266 comments, 1010 blanks, all 65199 lines

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)

## Files
| filename | language | code | comment | blank | total |
| :--- | :--- | ---: | ---: | ---: | ---: |
| [.idea/modules.xml](/.idea/modules.xml) | XML | 8 | 0 | 0 | 8 |
| [.idea/pathfinder.iml](/.idea/pathfinder.iml) | XML | 11 | 0 | 0 | 11 |
| [.idea/vcs.xml](/.idea/vcs.xml) | XML | 6 | 0 | 0 | 6 |
| [Cargo.lock](/Cargo.lock) | TOML | 4,592 | 2 | 480 | 5,074 |
| [Cargo.toml](/Cargo.toml) | TOML | 27 | 0 | 5 | 32 |
| [README.md](/README.md) | Markdown | 17 | 0 | 8 | 25 |
| [demos/staking-lst/README.md](/demos/staking-lst/README.md) | Markdown | 9 | 0 | 4 | 13 |
| [demos/staking-lst/config.yaml](/demos/staking-lst/config.yaml) | YAML | 0 | 0 | 1 | 1 |
| [scanner.log](/scanner.log) | Log | 57,014 | 0 | 5 | 57,019 |
| [src/bin/new.rs](/src/bin/new.rs) | Rust | 18 | 19 | 7 | 44 |
| [src/bin/test\_scanners1.rs](/src/bin/test_scanners1.rs) | Rust | 65 | 14 | 12 | 91 |
| [src/client\_asset\_scanner/mod.rs](/src/client_asset_scanner/mod.rs) | Rust | 4 | 1 | 2 | 7 |
| [src/lib.rs](/src/lib.rs) | Rust | 2 | 0 | 1 | 3 |
| [src/main.rs](/src/main.rs) | Rust | 3 | 0 | 1 | 4 |
| [src/new\_project\_scanner/alloy\_evm.rs](/src/new_project_scanner/alloy_evm.rs) | Rust | 246 | 7 | 41 | 294 |
| [src/new\_project\_scanner/config.rs](/src/new_project_scanner/config.rs) | Rust | 31 | 1 | 6 | 38 |
| [src/new\_project\_scanner/discovery.rs](/src/new_project_scanner/discovery.rs) | Rust | 65 | 5 | 14 | 84 |
| [src/new\_project\_scanner/errors.rs](/src/new_project_scanner/errors.rs) | Rust | 14 | 1 | 6 | 21 |
| [src/new\_project\_scanner/evm.rs](/src/new_project_scanner/evm.rs) | Rust | 48 | 0 | 12 | 60 |
| [src/new\_project\_scanner/filter.rs](/src/new_project_scanner/filter.rs) | Rust | 228 | 10 | 37 | 275 |
| [src/new\_project\_scanner/inspect.rs](/src/new_project_scanner/inspect.rs) | Rust | 88 | 7 | 17 | 112 |
| [src/new\_project\_scanner/mod.rs](/src/new_project_scanner/mod.rs) | Rust | 14 | 1 | 1 | 16 |
| [src/new\_project\_scanner/scanner.rs](/src/new_project_scanner/scanner.rs) | Rust | 96 | 5 | 18 | 119 |
| [src/new\_project\_scanner/store.rs](/src/new_project_scanner/store.rs) | Rust | 114 | 6 | 26 | 146 |
| [src/new\_project\_scanner/types.rs](/src/new_project_scanner/types.rs) | Rust | 133 | 4 | 30 | 167 |
| [src/optimizer/mod.rs](/src/optimizer/mod.rs) | Rust | 4 | 1 | 2 | 7 |
| [src/risk/apr.rs](/src/risk/apr.rs) | Rust | 348 | 24 | 62 | 434 |
| [src/risk/apr\_detector.rs](/src/risk/apr_detector.rs) | Rust | 0 | 1 | 1 | 2 |
| [src/risk/clickhouse.rs](/src/risk/clickhouse.rs) | Rust | 92 | 19 | 19 | 130 |
| [src/risk/clickhouse.sql](/src/risk/clickhouse.sql) | MS SQL | 65 | 8 | 20 | 93 |
| [src/risk/context.rs](/src/risk/context.rs) | Rust | 87 | 27 | 26 | 140 |
| [src/risk/convert.rs](/src/risk/convert.rs) | Rust | 45 | 2 | 12 | 59 |
| [src/risk/detectors/apr\_risk\_detector.rs](/src/risk/detectors/apr_risk_detector.rs) | Rust | 36 | 2 | 12 | 50 |
| [src/risk/detectors/holder\_concentration\_detector.rs](/src/risk/detectors/holder_concentration_detector.rs) | Rust | 31 | 11 | 14 | 56 |
| [src/risk/detectors/mod.rs](/src/risk/detectors/mod.rs) | Rust | 11 | 40 | 12 | 63 |
| [src/risk/detectors/owner\_privilege\_detector.rs](/src/risk/detectors/owner_privilege_detector.rs) | Rust | 52 | 6 | 15 | 73 |
| [src/risk/detectors/proxy\_admin\_eoa\_detector.rs](/src/risk/detectors/proxy_admin_eoa_detector.rs) | Rust | 25 | 5 | 8 | 38 |
| [src/risk/detectors/reward\_inflation\_detector.rs](/src/risk/detectors/reward_inflation_detector.rs) | Rust | 29 | 2 | 7 | 38 |
| [src/risk/detectors/tvl\_volatility\_detector.rs](/src/risk/detectors/tvl_volatility_detector.rs) | Rust | 31 | 3 | 8 | 42 |
| [src/risk/detectors/very\_new\_contract\_detector.rs](/src/risk/detectors/very_new_contract_detector.rs) | Rust | 22 | 1 | 5 | 28 |
| [src/risk/engine.rs](/src/risk/engine.rs) | Rust | 82 | 15 | 24 | 121 |
| [src/risk/mod.rs](/src/risk/mod.rs) | Rust | 9 | 1 | 1 | 11 |
| [src/risk/model.rs](/src/risk/model.rs) | Rust | 18 | 6 | 8 | 32 |
| [src/risk/scorer.rs](/src/risk/scorer.rs) | Rust | 45 | 5 | 9 | 59 |
| [src/risk/store.rs](/src/risk/store.rs) | Rust | 38 | 4 | 10 | 52 |
| [src/risk/types.rs](/src/risk/types.rs) | Rust | 0 | 0 | 1 | 1 |

[Summary](results.md) / Details / [Diff Summary](diff.md) / [Diff Details](diff-details.md)