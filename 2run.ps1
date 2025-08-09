# Ensure we're in the Rust project's root
$projectRoot = Split-Path $MyInvocation.MyCommand.Definition
Set-Location -Path $projectRoot

# Step 1: Build the project
cargo build

# Step 2: Start two cargo processes and capture their output without warnings
$procs = @()
$procs += Start-Process cargo -ArgumentList "run --quiet" -WorkingDirectory $projectRoot -RedirectStandardOutput "run1_out.txt" -RedirectStandardError "run1_err.txt" -NoNewWindow -PassThru
$procs += Start-Process cargo -ArgumentList "run --quiet" -WorkingDirectory $projectRoot -RedirectStandardOutput "run2_out.txt" -RedirectStandardError "run2_err.txt" -NoNewWindow -PassThru

# Also stream their stdout to the current process
foreach ($proc in $procs) {
    Start-Job -ScriptBlock {
        param($file)
        Get-Content -Path $file -Wait
    } -ArgumentList "$($proc.StartInfo.RedirectStandardOutput)"
}

Write-Host "Both cargo runs started. Press Ctrl+C to stop them."

# Trap Ctrl+C and kill both processes
$stopped = $false
Register-EngineEvent PowerShell.Exiting -Action {
    if (-not $stopped) {
        $global:stopped = $true
        foreach ($p in $procs) {
            try { $p.Kill() } catch {}
        }
    }
}

# Wait for both processes to exit or for Ctrl+C
while ($procs | Where-Object { !$_.HasExited }) {
    Start-Sleep -Seconds 1
}