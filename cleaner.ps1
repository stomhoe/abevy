param(
    [string[]] $Names = @("tilemap", "common", "dimension", "game_common")  # default if you run without args
)

function Remove-BuildArtifacts {
    param(
        [string[]] $Names
    )

    # Always include argentum_coop
    $allNames = $Names + "argentum_coop"

    # Build prefix list (include both with and without 'lib')
    $prefixes = @()
    foreach ($name in $allNames) {
        if ($name -like "lib*") {
            $prefixes += $name
            $prefixes += $name.Substring(3)  # also add without 'lib'
        } else {
            $prefixes += $name
            $prefixes += "lib$name"
        }
    }

    # Build regex pattern like ^(tilemap|libtilemap|argentum_coop|libargentum_coop)
    $pattern = "^(" + ($prefixes -join "|") + ")"

    Write-Host "Deleting build artifacts matching: $($prefixes -join ', ')"

    # Remove files in target\debug
    Get-ChildItem target\debug -Recurse -File |
        Where-Object { $_.Name -match $pattern } |
        Remove-Item -Force

    # Remove directories in target\debug\incremental
    $incPath = "target\debug\incremental"
    if (Test-Path $incPath) {
        Get-ChildItem $incPath -Directory |
            Where-Object { $_.Name -match $pattern } |
            Remove-Item -Recurse -Force
    }
}

# Run cleanup immediately with provided names
Remove-BuildArtifacts $Names
