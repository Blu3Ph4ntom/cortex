param(
    [string]$CortexBin = (Join-Path $PSScriptRoot "..\\target\\release\\cortex.exe"),
    [string]$WorkspaceRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path,
    [string]$FieldtestRoot = (Join-Path ([System.IO.Path]::GetTempPath()) "cortex-benchmarks"),
    [int]$ColdIndexIterations = 3,
    [int]$QueryIterations = 7,
    [int]$GrepIterations = 7,
    [string]$RepoKey,
    [switch]$UnsafeFullMatrix
)


function Ensure-Repo
{
    param(
        [string]$Name,
        [string]$Url,
        [string]$Path,
        [string]$Branch
    )

    if (-not $Url)
    {
        if (-not (Test-Path $Path))
        {
            throw "Benchmark repo missing at $Path (no URL configured for $Name)."
        }
        return
    }

    $markerPath = Join-Path $Path ".cortex-benchmark-repo"

    if (-not (Test-Path $Path))
    {
        Write-Host "Cloning $Name..."
        New-Item -ItemType Directory -Path (Split-Path $Path) -Force | Out-Null
        & git clone --quiet $Url $Path
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to clone $Name from $Url."
        }
        Set-Content -Path $markerPath -Value "Managed by scripts\benchmark.ps1"
    } elseif (-not (Test-Path (Join-Path $Path ".git")))
    {
        throw "Benchmark repo path is not a git repo: $Path"
    }

    $isOwned = Test-Path $markerPath
    $status = & git -C $Path status --porcelain
    if ($LASTEXITCODE -ne 0)
    {
        throw "Failed to read git status for $Name."
    }

    if (-not [string]::IsNullOrWhiteSpace($status) -and -not $isOwned)
    {
        throw "Repo $Name has uncommitted changes. Clean or stash before benchmarking."
    }

    if (-not [string]::IsNullOrWhiteSpace($status) -and $isOwned)
    {
        Write-Host "Resetting local changes in $Name (managed benchmark repo)."
        & git -C $Path reset --hard | Out-Null
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to reset $Name."
        }
        & git -C $Path clean -xdf | Out-Null
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to clean $Name."
        }
    }

    Write-Host "Updating $Name..."
    & git -C $Path fetch --quiet
    if ($LASTEXITCODE -ne 0)
    {
        throw "Failed to fetch $Name."
    }

    $targetBranch = $Branch
    if (-not $targetBranch)
    {
        $originHead = & git -C $Path symbolic-ref --quiet --short refs/remotes/origin/HEAD 2>$null
        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($originHead) -and $originHead -match '^origin/(.+)$')
        {
            $targetBranch = $Matches[1]
        }
    }

    if (-not $targetBranch)
    {
        $targetBranch = (& git -C $Path symbolic-ref --quiet --short HEAD 2>$null).Trim()
    }

    if ($targetBranch)
    {
        & git -C $Path show-ref --verify --quiet "refs/heads/$targetBranch"
        if ($LASTEXITCODE -ne 0)
        {
            & git -C $Path show-ref --verify --quiet "refs/remotes/origin/$targetBranch"
            if ($LASTEXITCODE -ne 0)
            {
                throw "Branch $targetBranch not found for $Name."
            }
            & git -C $Path checkout -B $targetBranch "origin/$targetBranch" | Out-Null
            if ($LASTEXITCODE -ne 0)
            {
                throw "Failed to check out $targetBranch for $Name."
            }
        } else
        {
            & git -C $Path checkout $targetBranch | Out-Null
            if ($LASTEXITCODE -ne 0)
            {
                throw "Failed to check out $targetBranch for $Name."
            }
        }
    }

    if ($isOwned -and $targetBranch)
    {
        & git -C $Path reset --hard "origin/$targetBranch" | Out-Null
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to reset $Name to origin/$targetBranch."
        }
    } elseif ($isOwned -and -not $targetBranch)
    {
        & git -C $Path reset --hard origin/HEAD | Out-Null
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to reset $Name to origin/HEAD."
        }
    } else
    {
        & git -C $Path pull --ff-only --quiet
        if ($LASTEXITCODE -ne 0)
        {
            throw "Failed to fast-forward $Name. Resolve local divergence before benchmarking."
        }
    }
}

function Remove-StorePath
{
    param([string]$StorePath)

    if ($StorePath -and (Test-Path $StorePath))
    {
        Remove-Item -Recurse -Force $StorePath
    }
}

function Clear-RepoStores
{
    param(
        [string]$RunStoresRoot,
        [string]$RepoKey
    )

    if (-not $RepoKey -or -not (Test-Path $RunStoresRoot))
    {
        return
    }

    Get-ChildItem -Path $RunStoresRoot -Filter "$RepoKey-*" -Directory -ErrorAction SilentlyContinue |
        ForEach-Object { Remove-Item -Recurse -Force $_.FullName }
}

function Reset-ManagedRepo
{
    param([string]$RepoPath)

    if (-not $RepoPath -or -not (Test-Path $RepoPath))
    {
        return
    }

    $markerPath = Join-Path $RepoPath ".cortex-benchmark-repo"
    if (-not (Test-Path $markerPath))
    {
        return
    }

    Write-Host "Cleaning managed benchmark repo artifacts in $RepoPath."
    & git -C $RepoPath reset --hard | Out-Null
    if ($LASTEXITCODE -ne 0)
    {
        throw "Failed to reset managed benchmark repo at $RepoPath."
    }
    & git -C $RepoPath clean -xdf | Out-Null
    if ($LASTEXITCODE -ne 0)
    {
        throw "Failed to clean managed benchmark repo at $RepoPath."
    }
}


function Get-Median
{
    param([double[]]$Values)

    if (-not $Values -or $Values.Count -eq 0)
    {
        return 0
    }

    $sorted = @($Values | ForEach-Object { [double]$_ } | Sort-Object { $_ })
    $count = $sorted.Count
    if ($count % 2 -eq 1)
    {
        return [math]::Round($sorted[[int]($count / 2)], 2)
    }

    return [math]::Round((($sorted[$count / 2 - 1] + $sorted[$count / 2]) / 2), 2)
}

function Invoke-Captured
{
    param(
        [string]$FilePath,
        [string[]]$Arguments
    )

    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $stdout = & $FilePath @Arguments 2>&1 | Out-String
    $exitCode = $LASTEXITCODE
    $stopwatch.Stop()

    if ($exitCode -ne 0)
    {
        throw "Command failed with exit code ${exitCode}: $FilePath $($Arguments -join ' ')`n$stdout"
    }

    [pscustomobject]@{
        duration_ms = [math]::Round($stopwatch.Elapsed.TotalMilliseconds, 2)
        stdout      = $stdout.Trim()
    }
}

function Invoke-CortexJson
{
    param(
        [string]$RepoPath,
        [string]$StorePath,
        [string[]]$Arguments
    )

    if (-not $Arguments -or $Arguments.Count -eq 0)
    {
        throw "Invoke-CortexJson requires a cortex subcommand."
    }

    $command = $Arguments[0]
    $rest = @()
    if ($Arguments.Count -gt 1)
    {
        $rest = $Arguments[1..($Arguments.Count - 1)]
    }

    $captured = Invoke-Captured -FilePath $CortexBin -Arguments (@($command, "--repo", $RepoPath, "--store-path", $StorePath) + $rest)
    [pscustomobject]@{
        duration_ms = $captured.duration_ms
        parsed      = ($captured.stdout | ConvertFrom-Json)
        raw         = $captured.stdout
    }
}

function Invoke-CortexIndex
{
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

function Invoke-GitGrep
{
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

function Get-QuerySummary
{
    param(
        [string]$Kind,
        [string]$Target,
        $Parsed
    )

    switch ($Kind)
    {
        "find_symbol"
        {
            $matches = @($Parsed)
            $top = if ($matches.Count -gt 0)
            { $matches[0]
            } else
            { $null
            }
            return [pscustomobject]@{
                result_count = $matches.Count
                answer_path   = if ($top -and $top.path)
                { [string]$top.path
                } else
                { ""
                }
                answer_line   = if ($top -and $top.span)
                { $top.span.start_line
                } else
                { 0
                }
                answer        = if ($top -and $top.path)
                {
                    $path = [string]$top.path
                    $line = if ($top.span)
                    { $top.span.start_line
                    } else
                    { 0
                    }
                    "${path}:$line"
                } else
                {
                    ""
                }
            }
        }
        "callers"
        {
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
        default
        {
            throw "Unsupported benchmark query kind: $Kind"
        }
    }
}

function Get-RelativePath
{
    param(
        [string]$BasePath,
        [string]$FullPath
    )

    if ([string]::IsNullOrWhiteSpace($FullPath))
    {
        return ""
    }

    $baseUri = [System.Uri]((Resolve-Path $BasePath).Path.TrimEnd('\') + '\')
    $fileUri = [System.Uri]((Resolve-Path $FullPath).Path)
    [System.Uri]::UnescapeDataString($baseUri.MakeRelativeUri($fileUri).ToString()).Replace('/', '\')
}

$benchRoot = Join-Path $WorkspaceRoot "benchmarks"
$siteDataRoot = Join-Path $WorkspaceRoot "site\\data"
$runStoresRoot = Join-Path $FieldtestRoot "benchstores\\runs"
$repoRoot = Join-Path $benchRoot "repos"

New-Item -ItemType Directory -Path $benchRoot -Force | Out-Null
New-Item -ItemType Directory -Path $siteDataRoot -Force | Out-Null
New-Item -ItemType Directory -Path $runStoresRoot -Force | Out-Null
New-Item -ItemType Directory -Path $repoRoot -Force | Out-Null

$repoSpecs = @(
    [pscustomobject]@{ key = "cortex"; name = "Cortex"; language = "Rust"; path = $WorkspaceRoot; url = $null; branch = $null },
    [pscustomobject]@{ key = "mini_redis"; name = "mini-redis"; language = "Rust"; repo_dir = "mini-redis"; url = "https://github.com/tokio-rs/mini-redis.git"; branch = $null },
    [pscustomobject]@{ key = "requests"; name = "requests"; language = "Python"; repo_dir = "requests"; url = "https://github.com/psf/requests.git"; branch = $null },
    [pscustomobject]@{ key = "chi"; name = "chi"; language = "Go"; repo_dir = "chi"; url = "https://github.com/go-chi/chi.git"; branch = $null },
    [pscustomobject]@{ key = "axios"; name = "axios"; language = "JavaScript"; repo_dir = "axios"; url = "https://github.com/axios/axios.git"; branch = $null },
    [pscustomobject]@{ key = "redux"; name = "redux"; language = "TypeScript"; repo_dir = "redux"; url = "https://github.com/reduxjs/redux.git"; branch = $null },
    [pscustomobject]@{ key = "retrofit"; name = "retrofit"; language = "Java"; repo_dir = "retrofit"; url = "https://github.com/square/retrofit.git"; branch = $null },
    [pscustomobject]@{ key = "kotlinx_coroutines"; name = "kotlinx.coroutines"; language = "Kotlin"; repo_dir = "kotlinx.coroutines"; url = "https://github.com/Kotlin/kotlinx.coroutines.git"; branch = $null },
    [pscustomobject]@{ key = "newtonsoft_json"; name = "Newtonsoft.Json"; language = "C#"; repo_dir = "Newtonsoft.Json"; url = "https://github.com/JamesNK/Newtonsoft.Json.git"; branch = $null },
    [pscustomobject]@{ key = "curl"; name = "curl"; language = "C"; repo_dir = "curl"; url = "https://github.com/curl/curl.git"; branch = $null },
    [pscustomobject]@{ key = "fmt"; name = "fmt"; language = "C++"; repo_dir = "fmt"; url = "https://github.com/fmtlib/fmt.git"; branch = $null },
    [pscustomobject]@{ key = "swift_arg_parser"; name = "swift-argument-parser"; language = "Swift"; repo_dir = "swift-argument-parser"; url = "https://github.com/apple/swift-argument-parser.git"; branch = $null },
    [pscustomobject]@{ key = "afnetworking"; name = "AFNetworking"; language = "Objective-C"; repo_dir = "AFNetworking"; url = "https://github.com/AFNetworking/AFNetworking.git"; branch = $null },
    [pscustomobject]@{ key = "jekyll"; name = "jekyll"; language = "Ruby"; repo_dir = "jekyll"; url = "https://github.com/jekyll/jekyll.git"; branch = $null },
    [pscustomobject]@{ key = "composer"; name = "composer"; language = "PHP"; repo_dir = "composer"; url = "https://github.com/composer/composer.git"; branch = $null },
    [pscustomobject]@{ key = "twitter_util"; name = "twitter-util"; language = "Scala"; repo_dir = "twitter-util"; url = "https://github.com/twitter/util.git"; branch = $null },
    [pscustomobject]@{ key = "phoenix"; name = "phoenix"; language = "Elixir"; repo_dir = "phoenix"; url = "https://github.com/phoenixframework/phoenix.git"; branch = $null },
    [pscustomobject]@{ key = "cowboy"; name = "cowboy"; language = "Erlang"; repo_dir = "cowboy"; url = "https://github.com/ninenines/cowboy.git"; branch = $null },
    [pscustomobject]@{ key = "dart_http"; name = "http"; language = "Dart"; repo_dir = "http"; url = "https://github.com/dart-lang/http.git"; branch = $null },
    [pscustomobject]@{ key = "luarocks"; name = "luarocks"; language = "Lua"; repo_dir = "luarocks"; url = "https://github.com/luarocks/luarocks.git"; branch = $null },
    [pscustomobject]@{ key = "dplyr"; name = "dplyr"; language = "R"; repo_dir = "dplyr"; url = "https://github.com/tidyverse/dplyr.git"; branch = $null },
    [pscustomobject]@{ key = "dataframes"; name = "DataFrames.jl"; language = "Julia"; repo_dir = "DataFrames.jl"; url = "https://github.com/JuliaData/DataFrames.jl.git"; branch = $null },
    [pscustomobject]@{ key = "aeson"; name = "aeson"; language = "Haskell"; repo_dir = "aeson"; url = "https://github.com/haskell/aeson.git"; branch = $null },
    [pscustomobject]@{ key = "ocaml_base"; name = "base"; language = "OCaml"; repo_dir = "base"; url = "https://github.com/janestreet/base.git"; branch = $null },
    [pscustomobject]@{ key = "clojure"; name = "clojure"; language = "Clojure"; repo_dir = "clojure"; url = "https://github.com/clojure/clojure.git"; branch = $null },
    [pscustomobject]@{ key = "nvm"; name = "nvm"; language = "Bash"; repo_dir = "nvm"; url = "https://github.com/nvm-sh/nvm.git"; branch = $null }
)

$repos = foreach ($spec in $repoSpecs)
{
    $path = if ($spec.path)
    { $spec.path
    } else
    { Join-Path $repoRoot $spec.repo_dir
    }
    [pscustomobject]@{
        key      = $spec.key
        name     = $spec.name
        language = $spec.language
        url      = $spec.url
        branch   = $spec.branch
        path     = $path
    }
}

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
        key        = "mini-redis-callers"
        repo_key   = "mini_redis"
        label      = "Trace read_frame callers"
        query_kind = "callers"
        target     = "read_frame"
        query_args = @("query", "callers", "--target", "read_frame")
        grep_text  = "read_frame"
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
    },
    [pscustomobject]@{
        key        = "redux-owner"
        repo_key   = "redux"
        label      = "Resolve redux.createStore owner"
        query_kind = "find_symbol"
        target     = "createStore"
        query_args = @("query", "find-symbol", "--name", "createStore")
        grep_text  = "createStore"
    },
    [pscustomobject]@{
        key        = "retrofit-owner"
        repo_key   = "retrofit"
        label      = "Resolve Retrofit owner"
        query_kind = "find_symbol"
        target     = "Retrofit"
        query_args = @("query", "find-symbol", "--name", "Retrofit")
        grep_text  = "Retrofit"
    },
    [pscustomobject]@{
        key        = "kotlinx-owner"
        repo_key   = "kotlinx_coroutines"
        label      = "Resolve CoroutineScope owner"
        query_kind = "find_symbol"
        target     = "CoroutineScope"
        query_args = @("query", "find-symbol", "--name", "CoroutineScope")
        grep_text  = "CoroutineScope"
    },
    [pscustomobject]@{
        key        = "newtonsoft-owner"
        repo_key   = "newtonsoft_json"
        label      = "Resolve JsonConvert owner"
        query_kind = "find_symbol"
        target     = "JsonConvert"
        query_args = @("query", "find-symbol", "--name", "JsonConvert")
        grep_text  = "JsonConvert"
    },
    [pscustomobject]@{
        key        = "curl-owner"
        repo_key   = "curl"
        label      = "Resolve curl_easy_init owner"
        query_kind = "find_symbol"
        target     = "curl_easy_init"
        query_args = @("query", "find-symbol", "--name", "curl_easy_init")
        grep_text  = "curl_easy_init"
    },
    [pscustomobject]@{
        key        = "fmt-owner"
        repo_key   = "fmt"
        label      = "Resolve fmt::format owner"
        query_kind = "find_symbol"
        target     = "format"
        query_args = @("query", "find-symbol", "--name", "format")
        grep_text  = "format"
    },
    [pscustomobject]@{
        key        = "swift-arg-owner"
        repo_key   = "swift_arg_parser"
        label      = "Resolve ArgumentParser owner"
        query_kind = "find_symbol"
        target     = "ArgumentParser"
        query_args = @("query", "find-symbol", "--name", "ArgumentParser")
        grep_text  = "ArgumentParser"
    },
    [pscustomobject]@{
        key        = "afnetworking-owner"
        repo_key   = "afnetworking"
        label      = "Resolve AFHTTPSessionManager owner"
        query_kind = "find_symbol"
        target     = "AFHTTPSessionManager"
        query_args = @("query", "find-symbol", "--name", "AFHTTPSessionManager")
        grep_text  = "AFHTTPSessionManager"
    },
    [pscustomobject]@{
        key        = "jekyll-owner"
        repo_key   = "jekyll"
        label      = "Resolve Jekyll owner"
        query_kind = "find_symbol"
        target     = "Jekyll"
        query_args = @("query", "find-symbol", "--name", "Jekyll")
        grep_text  = "Jekyll"
    },
    [pscustomobject]@{
        key        = "composer-owner"
        repo_key   = "composer"
        label      = "Resolve Composer owner"
        query_kind = "find_symbol"
        target     = "Composer"
        query_args = @("query", "find-symbol", "--name", "Composer")
        grep_text  = "Composer"
    },
    [pscustomobject]@{
        key        = "twitter-util-owner"
        repo_key   = "twitter_util"
        label      = "Resolve Future owner"
        query_kind = "find_symbol"
        target     = "Future"
        query_args = @("query", "find-symbol", "--name", "Future")
        grep_text  = "Future"
    },
    [pscustomobject]@{
        key        = "phoenix-owner"
        repo_key   = "phoenix"
        label      = "Resolve Phoenix.Endpoint owner"
        query_kind = "find_symbol"
        target     = "Endpoint"
        query_args = @("query", "find-symbol", "--name", "Endpoint")
        grep_text  = "Endpoint"
    },
    [pscustomobject]@{
        key        = "cowboy-owner"
        repo_key   = "cowboy"
        label      = "Resolve cowboy_req owner"
        query_kind = "find_symbol"
        target     = "cowboy_req"
        query_args = @("query", "find-symbol", "--name", "cowboy_req")
        grep_text  = "cowboy_req"
    },
    [pscustomobject]@{
        key        = "dart-http-owner"
        repo_key   = "dart_http"
        label      = "Resolve Client owner"
        query_kind = "find_symbol"
        target     = "Client"
        query_args = @("query", "find-symbol", "--name", "Client")
        grep_text  = "Client"
    },
    [pscustomobject]@{
        key        = "luarocks-owner"
        repo_key   = "luarocks"
        label      = "Resolve Rockspec owner"
        query_kind = "find_symbol"
        target     = "Rockspec"
        query_args = @("query", "find-symbol", "--name", "Rockspec")
        grep_text  = "Rockspec"
    },
    [pscustomobject]@{
        key        = "dplyr-owner"
        repo_key   = "dplyr"
        label      = "Resolve filter owner"
        query_kind = "find_symbol"
        target     = "filter"
        query_args = @("query", "find-symbol", "--name", "filter")
        grep_text  = "filter"
    },
    [pscustomobject]@{
        key        = "dataframes-owner"
        repo_key   = "dataframes"
        label      = "Resolve DataFrame owner"
        query_kind = "find_symbol"
        target     = "DataFrame"
        query_args = @("query", "find-symbol", "--name", "DataFrame")
        grep_text  = "DataFrame"
    },
    [pscustomobject]@{
        key        = "aeson-owner"
        repo_key   = "aeson"
        label      = "Resolve ToJSON owner"
        query_kind = "find_symbol"
        target     = "ToJSON"
        query_args = @("query", "find-symbol", "--name", "ToJSON")
        grep_text  = "ToJSON"
    },
    [pscustomobject]@{
        key        = "ocaml-base-owner"
        repo_key   = "ocaml_base"
        label      = "Resolve Base owner"
        query_kind = "find_symbol"
        target     = "Base"
        query_args = @("query", "find-symbol", "--name", "Base")
        grep_text  = "Base"
    },
    [pscustomobject]@{
        key        = "clojure-owner"
        repo_key   = "clojure"
        label      = "Resolve clojure.core owner"
        query_kind = "find_symbol"
        target     = "defn"
        query_args = @("query", "find-symbol", "--name", "defn")
        grep_text  = "defn"
    },
    [pscustomobject]@{
        key        = "nvm-owner"
        repo_key   = "nvm"
        label      = "Resolve nvm owner"
        query_kind = "find_symbol"
        target     = "nvm"
        query_args = @("query", "find-symbol", "--name", "nvm")
        grep_text  = "nvm"
    }
)

$availableKeys = @($repos | ForEach-Object { $_.key } | Sort-Object)
$availableKeysText = $availableKeys -join ", "

if ($RepoKey -and $UnsafeFullMatrix)
{
    throw "Choose either -RepoKey for a single repo run or -UnsafeFullMatrix for the full local matrix, not both."
}

if (-not $RepoKey -and -not $UnsafeFullMatrix)
{
    throw "Explicit repo selection required. Use -RepoKey <key> for a single repo run, or -UnsafeFullMatrix for the full local matrix. Available repo keys: $availableKeysText"
}

$selectedRepos = if ($RepoKey)
{ $repos | Where-Object { $_.key -eq $RepoKey }
} else
{ $repos
}

if (-not $selectedRepos -or $selectedRepos.Count -eq 0)
{
    throw "Unknown repo key '$RepoKey'. Available repo keys: $availableKeysText"
}

if (-not (Test-Path $CortexBin))
{
    throw "Cortex binary not found at $CortexBin. Build release first."
}

$repoBenchmarks = @()
$scenarioBenchmarks = @()

foreach ($repo in $selectedRepos)
{
    $activeStore = Join-Path $runStoresRoot "$($repo.key)-active"

    try
    {
        Ensure-Repo -Name $repo.name -Url $repo.url -Path $repo.path -Branch $repo.branch

        if (-not (Test-Path $repo.path))
        {
            throw "Benchmark repo missing: $($repo.path)"
        }

        $indexRuns = @()
        $statsSample = $null

        for ($i = 0; $i -lt $ColdIndexIterations; $i++)
        {
            $storePath = Join-Path $runStoresRoot "$($repo.key)-cold-$i"
            Remove-StorePath -StorePath $storePath

            $run = Invoke-CortexIndex -RepoPath $repo.path -StorePath $storePath
            $indexRuns += $run.duration_ms
            $statsSample = $run.parsed

            Remove-StorePath -StorePath $storePath
        }

        Remove-StorePath -StorePath $activeStore

        $activeBuild = Invoke-CortexIndex -RepoPath $repo.path -StorePath $activeStore
        $statsSample = $activeBuild.parsed

        $repoInfo = [pscustomobject]@{
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

        $repoBenchmarks += $repoInfo

        $repoScenarios = $scenarios | Where-Object { $_.repo_key -eq $repo.key }
        foreach ($scenario in $repoScenarios)
        {
            $queryRuns = @()
            $grepRuns = @()
            $parsedQuery = $null
            $rawQuery = ""
            $grepSample = $null

            for ($i = 0; $i -lt $QueryIterations; $i++)
            {
                $queryRun = Invoke-CortexJson -RepoPath $repoInfo.repo_path -StorePath $repoInfo.query_store_path -Arguments $scenario.query_args
                $queryRuns += $queryRun.duration_ms
                $parsedQuery = $queryRun.parsed
                $rawQuery = $queryRun.raw
            }

            for ($i = 0; $i -lt $GrepIterations; $i++)
            {
                $grepRun = Invoke-GitGrep -RepoPath $repoInfo.repo_path -Pattern $scenario.grep_text
                $grepRuns += $grepRun.duration_ms
                $grepSample = $grepRun
            }

            $summary = Get-QuerySummary -Kind $scenario.query_kind -Target $scenario.target -Parsed $parsedQuery
            $relativeAnswer = $summary.answer
            if ($scenario.query_kind -eq "find_symbol" -and $summary.answer_path)
            {
                $relativePath = Get-RelativePath -BasePath $repoInfo.repo_path -FullPath $summary.answer_path
                $relativeAnswer = if ($summary.answer_line)
                { "${relativePath}:$($summary.answer_line)"
                } else
                { $relativePath
                }
            }

            $candidateReduction = 0
            if ($summary.result_count -gt 0)
            {
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
    } finally
    {
        Clear-RepoStores -RunStoresRoot $runStoresRoot -RepoKey $repo.key
        Reset-ManagedRepo -RepoPath $repo.path
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
if ($totalCortexResults -gt 0)
{
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

foreach ($repo in $repoBenchmarks)
{
    $lines += "| $($repo.name) | $($repo.language) | $($repo.file_count) | $($repo.symbol_count) | $($repo.edge_count) | $($repo.index_median_ms) |"
}

$lines += ""
$lines += "## Structural Query Scenarios"
$lines += ""
$lines += "| Scenario | Cortex median (ms) | Baseline median (ms) | Cortex results | Grep hits | Grep files | Candidate reduction |"
$lines += "| --- | ---: | ---: | ---: | ---: | ---: | ---: |"

foreach ($scenario in $scenarioBenchmarks)
{
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
