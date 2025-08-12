
Get-ChildItem -Path $projectRoot -Filter "rustc*.txt" | Remove-Item -Force


$projectRoot = Split-Path $MyInvocation.MyCommand.Definition
Set-Location -Path $projectRoot

cargo run 2>&1 | Tee-Object -FilePath onerun_out.txt