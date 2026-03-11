$ErrorActionPreference = "Stop"

$repo = "Blu3Ph4ntom/cortex"
$installDir = if ($env:CORTEX_INSTALL_DIR) { $env:CORTEX_INSTALL_DIR } else { Join-Path $HOME ".cortex\bin" }

$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { throw "unsupported architecture" }
$artifact = "cortex-windows-$arch.zip"
$apiUrl = "https://api.github.com/repos/$repo/releases/latest"

$tmpRoot = Join-Path ([IO.Path]::GetTempPath()) ("cortex-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

try {
    $release = Invoke-RestMethod -Uri $apiUrl
    $asset = $release.assets | Where-Object { $_.name -eq $artifact } | Select-Object -First 1
    if (-not $asset) {
        throw "release artifact not found: $artifact"
    }

    $zipPath = Join-Path $tmpRoot $artifact
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath

    $extractPath = Join-Path $tmpRoot "extract"
    Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force

    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    Copy-Item (Join-Path $extractPath "cortex.exe") (Join-Path $installDir "cortex.exe") -Force
    Copy-Item (Join-Path $extractPath "cortexd.exe") (Join-Path $installDir "cortexd.exe") -Force

    Write-Host "installed cortex and cortexd to $installDir"
    Write-Host "add $installDir to PATH if needed"
}
finally {
    Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
}
