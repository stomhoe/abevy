param(
    [Alias("dir")]
    [string]$Directory = "."
)

# Resolve full path
$FullPath = Resolve-Path $Directory

# Get all files recursively
$files = Get-ChildItem -Path $FullPath -Recurse -File

foreach ($file in $files) {
    # Remove " copy" from the filename
    $newName = $file.Name -replace " copy", ""
    
    # Skip if nothing changed
    if ($newName -eq $file.Name) { continue }

    $targetPath = Join-Path $file.DirectoryName $newName

    # Extract base name and extension
    $baseName = [System.IO.Path]::GetFileNameWithoutExtension($newName)
    $ext = [System.IO.Path]::GetExtension($newName)

    # Loop until we find a unique name
    while (Test-Path $targetPath) {
        if ($baseName -match "^(.*?)(\d+)$") {
            # Ends with a number, increment it
            $prefix = $matches[1]
            $number = [int]$matches[2] + 1
            $baseName = "$prefix$number"
        } else {
            # Doesn't end with a number, prepend 1
            $baseName = "1$baseName"
        }
        $targetPath = Join-Path $file.DirectoryName ("$baseName$ext")
    }

    Rename-Item -Path $file.FullName -NewName (Split-Path $targetPath -Leaf)
    Write-Host "Renamed: $($file.Name) -> $(Split-Path $targetPath -Leaf)"
}
