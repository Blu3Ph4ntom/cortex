$ErrorActionPreference = "Stop"

$repo = "Blu3Ph4ntom/cortex"
$installDirOverridden = -not [string]::IsNullOrWhiteSpace($env:CORTEX_INSTALL_DIR)
$installDir = if ($installDirOverridden) { $env:CORTEX_INSTALL_DIR } else { Join-Path $HOME ".cortex\bin" }

$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { throw "unsupported architecture" }
$artifact = "cortex-windows-$arch.zip"
$apiUrl = "https://api.github.com/repos/$repo/releases/latest"

$tmpRoot = Join-Path ([IO.Path]::GetTempPath()) ("cortex-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmpRoot | Out-Null

function Get-PathEntries([string] $value) {
    if ([string]::IsNullOrWhiteSpace($value)) {
        return @()
    }

    return $value.Split(';', [System.StringSplitOptions]::RemoveEmptyEntries) |
        ForEach-Object { $_.Trim().TrimEnd('\') } |
        Where-Object { $_ }
}

function Add-ToSessionPath([string] $dir) {
    $normalizedDir = $dir.TrimEnd('\')
    $sessionEntries = Get-PathEntries $env:PATH
    if ($sessionEntries -contains $normalizedDir) {
        return $false
    }

    $env:PATH = if ([string]::IsNullOrWhiteSpace($env:PATH)) {
        $dir
    }
    else {
        "$dir;$env:PATH"
    }

    return $true
}

function Add-ToUserPath([string] $dir) {
    $normalizedDir = $dir.TrimEnd('\')
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $userEntries = Get-PathEntries $userPath
    if ($userEntries -contains $normalizedDir) {
        return "already-present"
    }

    $newUserPath = if ([string]::IsNullOrWhiteSpace($userPath)) {
        $dir
    }
    else {
        "$userPath;$dir"
    }

    [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
    return "added"
}

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

    $addedToSessionPath = Add-ToSessionPath $installDir
    $userPathStatus = if ($installDirOverridden) { "skipped" } else { Add-ToUserPath $installDir }

    Write-Host "installed cortex and cortexd to $installDir"
    if ($addedToSessionPath) {
        Write-Host "added $installDir to PATH for this PowerShell session"
    }

    switch ($userPathStatus) {
        "added" {
            Write-Host "added $installDir to your user PATH for new shells"
        }
        "already-present" {
            Write-Host "$installDir is already present in your user PATH"
        }
        "skipped" {
            Write-Host "CORTEX_INSTALL_DIR was set, so user PATH was not modified"
        }
    }

    Write-Host "run cortex --help"
}
finally {
    Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
}
