param(
    [string]$Repo = ".",
    [string]$Toolchain = "stable-x86_64-pc-windows-msvc",
    [int]$Port = 8788
)

$ErrorActionPreference = "Stop"

$repoPath = (Resolve-Path $Repo).Path
$storePath = Join-Path $repoPath ".cortex-selftest"
if (Test-Path $storePath) {
    Remove-Item -Recurse -Force $storePath
}

Write-Host "Indexing repository..."
cargo +$Toolchain build | Out-Null
$cortexBin = Join-Path $repoPath "target\debug\cortex.exe"
$daemonBin = Join-Path $repoPath "target\debug\cortexd.exe"

if (-not (Test-Path $cortexBin)) {
    throw "Missing CLI binary at $cortexBin"
}
if (-not (Test-Path $daemonBin)) {
    throw "Missing daemon binary at $daemonBin"
}

Write-Host "Indexing repository..."
$indexJson = & $cortexBin index --repo $repoPath --store-path $storePath
$index = $indexJson | ConvertFrom-Json
if ($index.file_count -lt 1) {
    throw "Expected indexed files, got $($index.file_count)"
}

Write-Host "Running doctor..."
$doctorJson = & $cortexBin doctor --repo $repoPath --store-path $storePath
$doctor = $doctorJson | ConvertFrom-Json
if ($doctor.indexed_files -lt 1) {
    throw "Doctor reported no indexed files"
}

Write-Host "Finding main..."
$mainJson = & $cortexBin query --repo $repoPath --store-path $storePath find-symbol --name main
$mains = $mainJson | ConvertFrom-Json
if ($mains.Count -lt 1) {
    throw "Expected to find at least one main symbol"
}

Write-Host "Finding callers of open_session..."
$callersJson = & $cortexBin query --repo $repoPath --store-path $storePath callers --target open_session
$callers = $callersJson | ConvertFrom-Json
if ($callers.nodes.Count -lt 2) {
    throw "Expected caller graph to include callers and target"
}

Write-Host "Explaining RepositorySession..."
$explainJson = & $cortexBin query --repo $repoPath --store-path $storePath explain --target RepositorySession
$explain = $explainJson | ConvertFrom-Json
if (-not $explain.summary) {
    throw "Explain returned an empty summary"
}

Write-Host "Starting daemon..."
$daemon = Start-Process -FilePath $daemonBin -ArgumentList "--repo",$repoPath,"--store-path",$storePath,"--bind","127.0.0.1:$Port" -PassThru

try {
    Start-Sleep -Seconds 4
    $http = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/graph/find_symbol?name=RepositorySession"
    if ($http.Count -lt 1) {
        throw "Daemon query did not return RepositorySession"
    }
}
finally {
    if ($daemon -and -not $daemon.HasExited) {
        Stop-Process -Id $daemon.Id -Force
    }
}

Write-Host "Self-test passed."
