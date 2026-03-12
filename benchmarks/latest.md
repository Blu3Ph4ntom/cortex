# Cortex Benchmarks

- Generated: 2026-03-12T10:01:47Z
- Commit: `3e4a670`
- Binary: `target/release/cortex.exe`
- Baseline: `git grep -n -w`

## Repo Index Medians

| Repo | Lang | Files | Symbols | Edges | Cold index median (ms) |
| --- | --- | ---: | ---: | ---: | ---: |
| Cortex | Rust | 13 | 323 | 2914 | 324.01 |
| mini-redis | Rust | 27 | 249 | 2254 | 147.67 |
| requests | Python | 36 | 759 | 5373 | 286.48 |
| chi | Go | 74 | 433 | 5809 | 277.88 |
| axios | JavaScript | 193 | 2985 | 26340 | 777.7 |

## Structural Query Scenarios

| Scenario | Cortex median (ms) | Baseline median (ms) | Cortex results | Grep hits | Grep files | Candidate reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Resolve RepositorySession owner | 59.83 | 43.55 | 1 | 49 | 14 | 49x |
| Trace open_session callers | 45.41 | 42.5 | 4 | 28 | 10 | 7x |
| Resolve requests.Session owner | 68.5 | 45.63 | 1 | 140 | 10 | 140x |
| Trace read_frame callers | 46.82 | 43.68 | 2 | 7 | 4 | 3.5x |
| Resolve chi.NewRouter owner | 81.59 | 47.2 | 1 | 120 | 37 | 120x |
| Resolve axios.dispatchRequest owner | 180.34 | 42.2 | 1 | 5 | 3 | 5x |

## Notes

- Cortex is benchmarked on structural tasks. The baseline is raw text search, which is fast but returns unranked line hits instead of a structural answer.
- Candidate reduction shows how many raw grep hits an agent avoids inspecting when Cortex returns a smaller structural result set.
