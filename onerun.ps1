$projectRoot = Split-Path $MyInvocation.MyCommand.Definition
Set-Location -Path $projectRoot

Get-ChildItem -Path $projectRoot -Filter "rustc*.txt" | Remove-Item -Force

cargo run | Tee-Object -FilePath onerun_out.txt
