param(
    [Parameter(Mandatory = $true)]
    [string]$WorkspaceRoot,
    [Parameter(Mandatory = $true)]
    [string]$DistDir,
    [string]$ArtifactName = "game_app-windows-x86_64"
)

# Produces a portable Windows bundle that keeps the extracted app directory self-contained for smoke startup checks. (ref: DL-006)

$BinaryPath = Join-Path $WorkspaceRoot "target/release/game_app.exe"
$StagingDir = Join-Path $DistDir $ArtifactName
$ArchivePath = Join-Path $DistDir ("{0}.zip" -f $ArtifactName)

# The staged directory survives zip extraction as the single runnable package root. (ref: DL-006)
# Portable archives keep the staged app directory self-contained for direct extraction and smoke startup checks. (ref: DL-006)
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
Remove-Item $StagingDir -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $ArchivePath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null

Copy-Item -Path $BinaryPath -Destination (Join-Path $StagingDir "game_app.exe")
Copy-Item -Path (Join-Path $WorkspaceRoot "assets") -Destination (Join-Path $StagingDir "assets") -Recurse

Compress-Archive -Path $StagingDir -DestinationPath $ArchivePath -Force
Write-Output $ArchivePath
