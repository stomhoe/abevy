# Ensure we're in the Rust project's root
$projectRoot = Split-Path $MyInvocation.MyCommand.Definition
Set-Location -Path $projectRoot

cargo run 2>&1 | Tee-Object -FilePath onerun_out.txt