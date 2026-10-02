$env:PATH = "$env:USERPROFILE\.cargo\bin;D:\Code\msys64\ucrt64\bin;$env:PATH"
Set-Location $PSScriptRoot
npm.cmd run dev
