param(
    [Alias("dir")]
    [string]$Directory = ".",
    [Parameter(Mandatory=$true)]
    [Alias("suf")]
    [string]$PreSuffix,
    [Alias("dry")]
    [switch]$DryRun
)


# Resolve to absolute path (works for relative and absolute input)
$FullPath = Resolve-Path $Directory

# Get all .ron files recursively from the specified directory
$files = Get-ChildItem -Path $FullPath -Filter *.ron -Recurse | Where-Object {
    # Only include files without an existing pre-suffix
    ([System.IO.Path]::GetFileNameWithoutExtension($_.Name) -notmatch "\.")
}

if ($files.Count -eq 0) {
    Write-Host "No .ron files without a pre-suffix found in $FullPath"
    exit
}

Write-Host "Found $($files.Count) .ron file(s) that would be renamed with pre-suffix '$PreSuffix'."

if (-not $DryRun) {
    $confirmation = Read-Host "Do you want to proceed? (Y/N)"
    if ($confirmation -ne "Y" -and $confirmation -ne "y") {
        Write-Host "Operation cancelled."
        exit
    }
}

foreach ($file in $files) {
    $baseName = [System.IO.Path]::GetFileNameWithoutExtension($file.Name)
    $newName = "$baseName.$PreSuffix.ron"

    if ($DryRun) {
        Write-Host "[Dry Run] $($file.Name) -> $newName"
    } else {
        Rename-Item -Path $file.FullName -NewName $newName
        Write-Host "Renamed: $($file.Name) -> $newName"
    }
}
