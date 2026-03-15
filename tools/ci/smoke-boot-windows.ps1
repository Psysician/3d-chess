param(
    [Parameter(Mandatory = $true)]
    [string]$ArchivePath,
    [int]$StartupSeconds = 10
)

$TempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$SmokeDir = Join-Path $TempRoot ("game-app-smoke-{0}" -f [guid]::NewGuid().ToString("N"))
$Process = $null
New-Item -ItemType Directory -Path $SmokeDir -Force | Out-Null
try {
    Expand-Archive -Path $ArchivePath -DestinationPath $SmokeDir -Force
    $AppDir = Get-ChildItem $SmokeDir -Directory |
        Where-Object { Test-Path (Join-Path $_.FullName "game_app.exe") } |
        Select-Object -First 1
    if (-not $AppDir) {
        throw "archive did not extract to a packaged app directory containing game_app.exe"
    }
    $ExePath = Join-Path $AppDir.FullName "game_app.exe"

    # Surviving the timeout window is the packaged-boot check; early exit means startup failed.
    $Process = Start-Process -FilePath $ExePath -WorkingDirectory $AppDir.FullName -PassThru
    if ($Process.WaitForExit($StartupSeconds * 1000)) {
        throw "game_app exited early with code $($Process.ExitCode)"
    }
}
finally {
    if ($Process -and -not $Process.HasExited) {
        Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
    }
    Remove-Item $SmokeDir -Recurse -Force -ErrorAction SilentlyContinue
}
