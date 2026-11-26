# Points this PowerShell session at the tools scripts/bootstrap.ps1 installed into .tools/.
# Dot-source it from anywhere:  . .\scripts\env.ps1
$root = Split-Path -Parent $PSScriptRoot
$tools = Join-Path $root '.tools'
if (-not (Test-Path (Join-Path $tools 'cargo\bin\cargo.exe'))) {
    Write-Warning 'No .tools in this checkout yet: run .\scripts\bootstrap.ps1 first.'
    return
}
$env:RUSTUP_HOME = Join-Path $tools 'rustup'
$env:CARGO_HOME = Join-Path $tools 'cargo'
$prepend = @((Join-Path $tools 'cargo\bin'), (Join-Path $tools 'mingw64\bin'))
$env:Path = (($prepend + ($env:Path -split ';' | Where-Object { $_ -and ($prepend -notcontains $_) })) -join ';')
Remove-Variable root, tools, prepend
