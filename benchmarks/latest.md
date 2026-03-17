# Cortex Benchmarks

- Generated: 2026-03-17T17:06:38Z
- Commit: `a0a4892`
- Binary: `target/release/cortex`
- Baseline: `git grep -n -w`

## Repo Index Medians

| Repo | Lang | Files | Symbols | Edges | Cold index median (ms) |
| --- | --- | ---: | ---: | ---: | ---: |
| Cortex | Rust | 23 | 289 | 2669 | 178.84 |
| mini-redis | Rust | 28 | 303 | 2512 | 143.18 |
| requests | Python | 50 | 759 | 6595 | 263.83 |
| chi | Go | 76 | 433 | 5811 | 282.34 |
| axios | JavaScript | 298 | 4442 | 39501 | 1209.77 |
| redux | TypeScript | 223 | 16022 | 227388 | 7155.1 |
| retrofit | Java | 927 | 7770 | 101459 | 4355.44 |
| kotlinx.coroutines | Kotlin | 1098 | 78 | 1429 | 1043.47 |
| Newtonsoft.Json | C# | 947 | 8435 | 136192 | 6894.25 |
| curl | C | 1050 | 907 | 17178 | 2126.93 |
| fmt | C++ | 89 | 107 | 913 | 1327.34 |
| swift-argument-parser | Swift | 169 | 2067 | 11716 | 540.0 |
| AFNetworking | Objective-C | 88 | 22 | 171 | 222.5 |
| jekyll | Ruby | 295 | 94 | 1045 | 263.3 |
| composer | PHP | 589 | 4771 | 14347 | 1164.51 |
| twitter-util | Scala | 757 | 7155 | 83211 | 4203.7 |
| phoenix | Elixir | 215 | 1531 | 15950 | 732.59 |
| cowboy | Erlang | 193 | 0 | 193 | 372.47 |
| http | Dart | 410 | 351 | 2825 | 1988.99 |
| luarocks | Lua | 174 | 1004 | 5165 | 563.94 |
| dplyr | R | 217 | 1124 | 2575 | 555.2 |
| DataFrames.jl | Julia | 80 | 0 | 80 | 1007.6 |
| aeson | Haskell | 135 | 2 | 139 | 342.35 |
| base | OCaml | 597 | 39 | 769 | 914.59 |
| clojure | Clojure | 335 | 5298 | 47543 | 3307.14 |
| nvm | Bash | 24 | 158 | 340 | 118.37 |

## Structural Query Scenarios

| Scenario | Cortex median (ms) | Baseline median (ms) | Cortex results | Grep hits | Grep files | Candidate reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Resolve RepositorySession owner | 102.03 | 17.86 | 2 | 51 | 15 | 25.5x |
| Trace open_session callers | 90.12 | 23.81 | 4 | 28 | 11 | 7.0x |
| Trace read_frame callers | 85.32 | 17.71 | 2 | 7 | 4 | 3.5x |
| Resolve requests.Session owner | 148.44 | 55.95 | 1 | 140 | 10 | 140.0x |
| Resolve chi.NewRouter owner | 162.04 | 30.27 | 1 | 120 | 37 | 120.0x |
| Resolve axios.dispatchRequest owner | 564.23 | 33.28 | 1 | 5 | 3 | 5.0x |
| Resolve redux.createStore owner | 2882.81 | 47.69 | 1 | 333 | 55 | 333.0x |
| Resolve Retrofit owner | 1499.76 | 72.55 | 1 | 1691 | 498 | 1691.0x |
| Resolve CoroutineScope owner | 76.6 | 80.08 | 0 | 645 | 194 | 0.0x |
| Resolve JsonConvert owner | 2114.09 | 66.65 | 1 | 2061 | 211 | 2061.0x |
| Resolve curl_easy_init owner | 353.41 | 192.7 | 0 | 926 | 767 | 0.0x |
| Resolve fmt::format owner | 97.12 | 24.14 | 0 | 2581 | 92 | 0.0x |
| Resolve ArgumentParser owner | 216.7 | 27.01 | 0 | 204 | 116 | 0.0x |
| Resolve AFHTTPSessionManager owner | 57.05 | 24.28 | 0 | 88 | 16 | 0.0x |
| Resolve Jekyll owner | 67.95 | 53.06 | 0 | 2279 | 428 | 0.0x |
| Resolve Composer owner | 242.21 | 85.15 | 1 | 6428 | 625 | 6428.0x |
| Resolve Future owner | 1385.72 | 82.77 | 1 | 2745 | 172 | 2745.0x |
| Resolve Phoenix.Endpoint owner | 213.85 | 36.75 | 0 | 501 | 71 | 0.0x |
| Resolve cowboy_req owner | 54.01 | 37.61 | 0 | 1370 | 160 | 0.0x |
| Resolve Client owner | 290.86 | 48.62 | 1 | 250 | 60 | 250.0x |
| Resolve Rockspec owner | 195.73 | 46.61 | 0 | 152 | 52 | 0.0x |
| Resolve filter owner | 159.58 | 33.28 | 0 | 874 | 91 | 0.0x |
| Resolve DataFrame owner | 53.44 | 23.92 | 0 | 6804 | 77 | 0.0x |
| Resolve ToJSON owner | 53.55 | 72.95 | 0 | 391 | 38 | 0.0x |
| Resolve Base owner | 62.62 | 37.87 | 0 | 411 | 175 | 0.0x |
| Resolve clojure.core owner | 715.67 | 28.06 | 0 | 1526 | 107 | 0.0x |
| Resolve nvm owner | 55.24 | 31.91 | 1 | 1845 | 294 | 1845.0x |

## Notes

- Cortex is benchmarked on structural tasks. The baseline is raw text search, which is fast but returns unranked line hits instead of a structural answer.
- Candidate reduction shows how many raw grep hits an agent avoids inspecting when Cortex returns a smaller structural result set.