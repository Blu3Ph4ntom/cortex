# Cortex Benchmarks

- Generated: 2026-03-12T09:57:04Z
- Commit: `179a771`
- Binary: `target/release/cortex.exe`
- Baseline: `git grep -n -w`

## Repo Index Medians

| Repo | Lang | Files | Symbols | Edges | Cold index median (ms) |
| --- | --- | ---: | ---: | ---: | ---: |
| Cortex | Rust | 13 | 323 | 2914 | 591.41 |
| mini-redis | Rust | 27 | 249 | 2254 | 481.45 |
| requests | Python | 36 | 759 | 5373 | 714.83 |
| chi | Go | 74 | 433 | 5809 | 1806.68 |
| axios | JavaScript | 193 | 2985 | 26340 | 4945.19 |

## Structural Query Scenarios

| Scenario | Cortex median (ms) | Baseline median (ms) | Cortex results | Grep hits | Grep files | Candidate reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Resolve RepositorySession owner | 57.33 | 50.03 | 1 | 49 | 14 | 49x |
| Trace open_session callers | 52.7 | 40.94 | 4 | 28 | 10 | 7x |
| Resolve requests.Session owner | 73.54 | 44.71 | 1 | 140 | 10 | 140x |
| Trace read_frame callers | 46.19 | 40.21 | 2 | 7 | 4 | 3.5x |
| Resolve chi.NewRouter owner | 89.99 | 42.83 | 1 | 120 | 37 | 120x |
| Resolve axios.dispatchRequest owner | 192.12 | 45.03 | 1 | 5 | 3 | 5x |

## Notes

- Cortex is benchmarked on structural tasks. The baseline is raw text search, which is fast but returns unranked line hits instead of a structural answer.
- Candidate reduction shows how many raw grep hits an agent avoids inspecting when Cortex returns a smaller structural result set.
