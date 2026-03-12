# Cortex Benchmarks

- Generated: 2026-03-12T09:28:53Z
- Commit: `30ce799`
- Binary: `target/release/cortex.exe`
- Baseline: `git grep -n -w`

## Repo Index Medians

| Repo | Lang | Files | Symbols | Edges | Cold index median (ms) |
| --- | --- | ---: | ---: | ---: | ---: |
| Cortex | Rust | 11 | 249 | 2364 | 175.13 |
| mini-redis | Rust | 27 | 249 | 2254 | 104.74 |
| requests | Python | 36 | 759 | 5373 | 170.81 |
| chi | Go | 74 | 433 | 5809 | 206.96 |
| axios | JavaScript | 193 | 2985 | 26340 | 581.43 |

## Structural Query Scenarios

| Scenario | Cortex median (ms) | Baseline median (ms) | Cortex results | Grep hits | Grep files | Candidate reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Resolve RepositorySession owner | 40.4 | 40.98 | 1 | 45 | 10 | 45x |
| Trace open_session callers | 40.44 | 41.71 | 4 | 21 | 6 | 5.25x |
| Resolve requests.Session owner | 65.38 | 41.33 | 1 | 140 | 10 | 140x |
| Trace read_frame callers | 39.83 | 38.29 | 2 | 7 | 4 | 3.5x |
| Resolve chi.NewRouter owner | 70.33 | 39.38 | 1 | 120 | 37 | 120x |
| Resolve axios.dispatchRequest owner | 188.2 | 43.29 | 1 | 5 | 3 | 5x |

## Notes

- Cortex is benchmarked on structural tasks. The baseline is raw text search, which is fast but returns unranked line hits instead of a structural answer.
- Candidate reduction shows how many raw grep hits an agent avoids inspecting when Cortex returns a smaller structural result set.
