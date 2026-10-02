$ErrorActionPreference = "Stop"
$env:PATH = "$env:USERPROFILE\.cargo\bin;D:\Code\msys64\ucrt64\bin;$env:PATH"
Set-Location $PSScriptRoot
npm.cmd run build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
New-Item -ItemType Directory -Force release | Out-Null
Copy-Item src-tauri/target/release/Codex-QuotaWhale.exe release/Codex-QuotaWhale.exe
Copy-Item src-tauri/target/release/WebView2Loader.dll release/WebView2Loader.dll
New-Item -ItemType Directory -Force release/licenses | Out-Null
Copy-Item licenses/* release/licenses -Force
Copy-Item README.md release/README.md


