param(
    [string]$CortexBin = (Join-Path $PSScriptRoot "..\\target\\release\\cortex.exe"),
    [string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [string]$FieldtestRoot = "A:\\Projects\\cortex-fieldtests",
    [int]$ColdIndexIterations = 3,
    [int]$QueryIterations = 7,
    [int]$GrepIterations = 7
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-Median {
    param([double[]]$Values)

    if (-not $Values -or $Values.Count -eq 0) {
        return 0
    }

    $sorted = @($Values | ForEach-Object { [double]$_ } | Sort-Object { $_ })
    $count = $sorted.Count
    if ($count % 2 -eq 1) {
        return [math]::Round($sorted[[int]($count / 2)], 2)
    }

    return [math]::Round((($sorted[$count / 2 - 1] + $sorted[$count / 2]) / 2), 2)
}

function Invoke-Captured {
    param(
        [string]$FilePath,
        [string[]]$Arguments
    )

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $stdout = & $FilePath @Arguments 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    $stopwatch.Stop()

    if ($exitCode -ne 0) {
        throw "Command failed with exit code ${exitCode}: $FilePath $($Arguments -join ' ')`n$stdout"
    }

    [pscustomobject]@{
        duration_ms = [math]::Round($stopwatch.Elapsed.TotalMilliseconds, 2)
        stdout      = $stdout.Trim()
    }
}

function Invoke-CortexJson {
    param(
        [string]$RepoPath,
        [string]$StorePath,
        [string[]]$Arguments
    )

    if (-not $Arguments -or $Arguments.Count -eq 0) {
        throw "Invoke-CortexJson requires a cortex subcommand."
    }

    $command = $Arguments[0]
    $rest = @()
    if ($Arguments.Count -gt 1) {
        $rest = $Arguments[1..($Arguments.Count - 1)]
    }

    $captured = Invoke-Captured -FilePath $CortexBin -Arguments (@($command, "--repo", $RepoPath, "--store-path", $StorePath) + $rest)
    [pscustomobject]@{
        duration_ms = $captured.duration_ms
        parsed      = ($captured.stdout | ConvertFrom-Json)
        raw         = $captured.stdout
    }
}

function Invoke-CortexIndex {
    param(
        [string]$RepoPath,
        [string]$StorePath
    )

    $captured = Invoke-Captured -FilePath $CortexBin -Arguments @("index", "--repo", $RepoPath, "--store-path", $StorePath)
    [pscustomobject]@{
        duration_ms = $captured.duration_ms
        parsed      = ($captured.stdout | ConvertFrom-Json)
        raw         = $captured.stdout
    }
}

function Invoke-GitGrep {
    param(
        [string]$RepoPath,
        [string]$Pattern
    )

    $captured = Invoke-Captured -FilePath "git" -Arguments @("-C", $RepoPath, "grep", "-n", "-w", "--no-color", "-I", "-e", $Pattern)
    $lines = @($captured.stdout -split "(`r`n|`n|`r)" | Where-Object { $_.Trim() -ne "" })
    $uniqueFiles = @($lines | ForEach-Object {
            ($_ -split ":", 2)[0]
        } | Sort-Object -Unique)

    [pscustomobject]@{
        duration_ms  = $captured.duration_ms
        raw          = $captured.stdout
        hit_count    = $lines.Count
        file_count   = $uniqueFiles.Count
        output_chars = $captured.stdout.Length
    }
}

function Get-QuerySummary {
    param(
        [string]$Kind,
        [string]$Target,
        $Parsed
    )

    switch ($Kind) {
        "find_symbol" {
            $matches = @($Parsed)
            $top = if ($matches.Count -gt 0) { $matches[0] } else { $null }
            return [pscustomobject]@{
                result_count = $matches.Count
                answer_path   = if ($top -and $top.path) { [string]$top.path } else { "" }
                answer_line   = if ($top -and $top.span) { $top.span.start_line } else { 0 }
                answer        = if ($top -and $top.path) {
                    $path = [string]$top.path
                    $line = if ($top.span) { $top.span.start_line } else { 0 }
                    "${path}:$line"
                } else {
                    ""
                }
            }
        }
        "callers" {
            $callerNodes = @(
                $Parsed.nodes |
                Where-Object {
                    $_.kind -eq "Symbol" -and $_.name -ne $Target
                }
            )
            return [pscustomobject]@{
                result_count = $callerNodes.Count
                answer_path  = ""
                answer_line  = 0
                answer       = ($callerNodes | Select-Object -First 4 | ForEach-Object { $_.name }) -join ", "
            }
        }
        default {
            throw "Unsupported benchmark query kind: $Kind"
        }
    }
}

function Get-RelativePath {
    param(
        [string]$BasePath,
        [string]$FullPath
    )

    if ([string]::IsNullOrWhiteSpace($FullPath)) {
        return ""
    }

    $baseUri = [System.Uri]((Resolve-Path $BasePath).Path.TrimEnd('\') + '\')
    $fileUri = [System.Uri]((Resolve-Path $FullPath).Path)
    [System.Uri]::UnescapeDataString($baseUri.MakeRelativeUri($fileUri).ToString()).Replace('/', '\')
}

if (-not (Test-Path $CortexBin)) {
    throw "Cortex binary not found at $CortexBin. Build release first."
}

$benchRoot = Join-Path $WorkspaceRoot "benchmarks"
$siteDataRoot = Join-Path $WorkspaceRoot "site\\data"
$runStoresRoot = Join-Path $FieldtestRoot "benchstores\\runs"

New-Item -ItemType Directory -Path $benchRoot -Force | Out-Null
New-Item -ItemType Directory -Path $siteDataRoot -Force | Out-Null
New-Item -ItemType Directory -Path $runStoresRoot -Force | Out-Null

$repos = @(
    [pscustomobject]@{ key = "cortex"; name = "Cortex"; language = "Rust"; path = $WorkspaceRoot },
    [pscustomobject]@{ key = "mini_redis"; name = "mini-redis"; language = "Rust"; path = (Join-Path $FieldtestRoot "mini-redis") },
    [pscustomobject]@{ key = "requests"; name = "requests"; language = "Python"; path = (Join-Path $FieldtestRoot "requests") },
    [pscustomobject]@{ key = "chi"; name = "chi"; language = "Go"; path = (Join-Path $FieldtestRoot "chi") },
    [pscustomobject]@{ key = "axios"; name = "axios"; language = "JavaScript"; path = (Join-Path $FieldtestRoot "axios") }
)

$scenarios = @(
    [pscustomobject]@{
        key        = "cortex-owner"
        repo_key   = "cortex"
        label      = "Resolve RepositorySession owner"
        query_kind = "find_symbol"
        target     = "RepositorySession"
        query_args = @("query", "find-symbol", "--name", "RepositorySession")
        grep_text  = "RepositorySession"
    },
    [pscustomobject]@{
        key        = "cortex-callers"
        repo_key   = "cortex"
        label      = "Trace open_session callers"
        query_kind = "callers"
        target     = "open_session"
        query_args = @("query", "callers", "--target", "open_session")
        grep_text  = "open_session"
    },
    [pscustomobject]@{
        key        = "requests-owner"
        repo_key   = "requests"
        label      = "Resolve requests.Session owner"
        query_kind = "find_symbol"
        target     = "Session"
        query_args = @("query", "find-symbol", "--name", "Session")
        grep_text  = "Session"
    },
    [pscustomobject]@{
        key        = "mini-redis-callers"
        repo_key   = "mini_redis"
        label      = "Trace read_frame callers"
        query_kind = "callers"
        target     = "read_frame"
        query_args = @("query", "callers", "--target", "read_frame")
        grep_text  = "read_frame"
    },
    [pscustomobject]@{
        key        = "chi-owner"
        repo_key   = "chi"
        label      = "Resolve chi.NewRouter owner"
        query_kind = "find_symbol"
        target     = "NewRouter"
        query_args = @("query", "find-symbol", "--name", "NewRouter")
        grep_text  = "NewRouter"
    },
    [pscustomobject]@{
        key        = "axios-owner"
        repo_key   = "axios"
        label      = "Resolve axios.dispatchRequest owner"
        query_kind = "find_symbol"
        target     = "dispatchRequest"
        query_args = @("query", "find-symbol", "--name", "dispatchRequest")
        grep_text  = "dispatchRequest"
    }
)

$repoBenchmarks = @()
$activeStores = @{}

foreach ($repo in $repos) {
    if (-not (Test-Path $repo.path)) {
        throw "Benchmark repo missing: $($repo.path)"
    }

    $indexRuns = @()
    $statsSample = $null

    for ($i = 0; $i -lt $ColdIndexIterations; $i++) {
        $storePath = Join-Path $runStoresRoot "$($repo.key)-cold-$i"
        if (Test-Path $storePath) {
            Remove-Item -Recurse -Force $storePath
        }

        $run = Invoke-CortexIndex -RepoPath $repo.path -StorePath $storePath
        $indexRuns += $run.duration_ms
        $statsSample = $run.parsed

        Remove-Item -Recurse -Force $storePath
    }

    $activeStore = Join-Path $runStoresRoot "$($repo.key)-active"
    if (Test-Path $activeStore) {
        Remove-Item -Recurse -Force $activeStore
    }

    $activeBuild = Invoke-CortexIndex -RepoPath $repo.path -StorePath $activeStore
    $statsSample = $activeBuild.parsed
    $activeStores[$repo.key] = $activeStore

    $repoBenchmarks += [pscustomobject]@{
        key               = $repo.key
        name              = $repo.name
        language          = $repo.language
        repo_path         = $repo.path
        index_median_ms   = (Get-Median -Values $indexRuns)
        index_run_ms      = $indexRuns
        file_count        = $statsSample.file_count
        symbol_count      = $statsSample.symbol_count
        edge_count        = $statsSample.edge_count
        query_store_path  = $activeStore
    }
}

$scenarioBenchmarks = @()
foreach ($scenario in $scenarios) {
    $repoInfo = $repoBenchmarks | Where-Object { $_.key -eq $scenario.repo_key } | Select-Object -First 1
    if (-not $repoInfo) {
        throw "Missing repo benchmark for scenario $($scenario.key)"
    }

    $queryRuns = @()
    $grepRuns = @()
    $parsedQuery = $null
    $rawQuery = ""
    $grepSample = $null

    for ($i = 0; $i -lt $QueryIterations; $i++) {
        $queryRun = Invoke-CortexJson -RepoPath $repoInfo.repo_path -StorePath $repoInfo.query_store_path -Arguments $scenario.query_args
        $queryRuns += $queryRun.duration_ms
        $parsedQuery = $queryRun.parsed
        $rawQuery = $queryRun.raw
    }

    for ($i = 0; $i -lt $GrepIterations; $i++) {
        $grepRun = Invoke-GitGrep -RepoPath $repoInfo.repo_path -Pattern $scenario.grep_text
        $grepRuns += $grepRun.duration_ms
        $grepSample = $grepRun
    }

    $summary = Get-QuerySummary -Kind $scenario.query_kind -Target $scenario.target -Parsed $parsedQuery
    $relativeAnswer = $summary.answer
    if ($scenario.query_kind -eq "find_symbol" -and $summary.answer_path) {
        $relativePath = Get-RelativePath -BasePath $repoInfo.repo_path -FullPath $summary.answer_path
        $relativeAnswer = if ($summary.answer_line) { "${relativePath}:$($summary.answer_line)" } else { $relativePath }
    }

    $candidateReduction = 0
    if ($summary.result_count -gt 0) {
        $candidateReduction = [math]::Round($grepSample.hit_count / $summary.result_count, 2)
    }

    $scenarioBenchmarks += [pscustomobject]@{
        key                        = $scenario.key
        repo_key                   = $scenario.repo_key
        repo_name                  = $repoInfo.name
        language                   = $repoInfo.language
        label                      = $scenario.label
        query_kind                 = $scenario.query_kind
        target                     = $scenario.target
        cortex_median_ms           = (Get-Median -Values $queryRuns)
        cortex_run_ms              = $queryRuns
        cortex_result_count        = $summary.result_count
        cortex_answer              = $relativeAnswer
        cortex_output_chars        = $rawQuery.Length
        baseline_tool              = "git grep -n -w"
        baseline_median_ms         = (Get-Median -Values $grepRuns)
        baseline_run_ms            = $grepRuns
        baseline_hit_count         = $grepSample.hit_count
        baseline_file_count        = $grepSample.file_count
        baseline_output_chars      = $grepSample.output_chars
        candidate_reduction_factor = $candidateReduction
    }
}

foreach ($storePath in $activeStores.Values) {
    if (Test-Path $storePath) {
        Remove-Item -Recurse -Force $storePath
    }
}

$timestamp = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
$commit = (git -C $WorkspaceRoot rev-parse --short HEAD).Trim()
$repoCount = $repoBenchmarks.Count
$scenarioCount = $scenarioBenchmarks.Count
$corpusFiles = ($repoBenchmarks | Measure-Object -Property file_count -Sum).Sum
$corpusSymbols = ($repoBenchmarks | Measure-Object -Property symbol_count -Sum).Sum
$corpusEdges = ($repoBenchmarks | Measure-Object -Property edge_count -Sum).Sum
$totalCortexResults = ($scenarioBenchmarks | Measure-Object -Property cortex_result_count -Sum).Sum
$totalGrepHits = ($scenarioBenchmarks | Measure-Object -Property baseline_hit_count -Sum).Sum
$totalGrepFiles = ($scenarioBenchmarks | Measure-Object -Property baseline_file_count -Sum).Sum
$overallReduction = 0
if ($totalCortexResults -gt 0) {
    $overallReduction = [math]::Round($totalGrepHits / $totalCortexResults, 2)
}

$payload = [pscustomobject]@{
    generated_at = $timestamp
    commit       = $commit
    summary      = [pscustomobject]@{
        repo_count                    = $repoCount
        scenario_count                = $scenarioCount
        corpus_files                  = $corpusFiles
        corpus_symbols                = $corpusSymbols
        corpus_edges                  = $corpusEdges
        total_cortex_results          = $totalCortexResults
        total_baseline_hits           = $totalGrepHits
        total_baseline_files          = $totalGrepFiles
        overall_candidate_reduction   = $overallReduction
    }
    methodology  = [pscustomobject]@{
        benchmark_host        = $env:COMPUTERNAME
        cold_index_iterations = $ColdIndexIterations
        warm_query_iterations = $QueryIterations
        baseline_iterations   = $GrepIterations
        baseline              = "git grep -n -w"
        binary                = "target/release/cortex.exe"
        note                  = "Cold index medians come from fresh stores. Query medians reuse a warm local store and are compared against raw repo text search."
    }
    repos        = $repoBenchmarks
    scenarios    = $scenarioBenchmarks
}

$json = $payload | ConvertTo-Json -Depth 8
$jsonPath = Join-Path $benchRoot "latest.json"
$siteJsonPath = Join-Path $siteDataRoot "benchmarks.json"

Set-Content -Path $jsonPath -Value $json
Set-Content -Path $siteJsonPath -Value $json

$lines = @(
    "# Cortex Benchmarks",
    "",
    "- Generated: $timestamp",
    "- Commit: ``$commit``",
    "- Binary: ``target/release/cortex.exe``",
    "- Baseline: ``git grep -n -w``",
    "",
    "## Repo Index Medians",
    "",
    "| Repo | Lang | Files | Symbols | Edges | Cold index median (ms) |",
    "| --- | --- | ---: | ---: | ---: | ---: |"
)

foreach ($repo in $repoBenchmarks) {
    $lines += "| $($repo.name) | $($repo.language) | $($repo.file_count) | $($repo.symbol_count) | $($repo.edge_count) | $($repo.index_median_ms) |"
}

$lines += ""
$lines += "## Structural Query Scenarios"
$lines += ""
$lines += "| Scenario | Cortex median (ms) | Baseline median (ms) | Cortex results | Grep hits | Grep files | Candidate reduction |"
$lines += "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"

foreach ($scenario in $scenarioBenchmarks) {
    $lines += "| $($scenario.label) | $($scenario.cortex_median_ms) | $($scenario.baseline_median_ms) | $($scenario.cortex_result_count) | $($scenario.baseline_hit_count) | $($scenario.baseline_file_count) | $($scenario.candidate_reduction_factor)x |"
}

$lines += ""
$lines += "## Notes"
$lines += ""
$lines += "- Cortex is benchmarked on structural tasks. The baseline is raw text search, which is fast but returns unranked line hits instead of a structural answer."
$lines += "- Candidate reduction shows how many raw grep hits an agent avoids inspecting when Cortex returns a smaller structural result set."

$markdownPath = Join-Path $benchRoot "latest.md"
Set-Content -Path $markdownPath -Value ($lines -join "`r`n")

Write-Host "Wrote benchmark artifacts:"
Write-Host "  $jsonPath"
Write-Host "  $siteJsonPath"
Write-Host "  $markdownPath"
