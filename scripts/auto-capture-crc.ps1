param(
    [string]$ProfileName = "",
    [int]$TimeoutSeconds = 45,
    [string]$TpDir = "C:\Users\robon\Desktop\TPBootstrapper",
    [string]$DllPath = "E:\Projects\OutputBlaster\bin\x86\Release\OutputBlaster.dll",
    [string]$CrcFile = "C:\Users\robon\AppData\Local\Temp\ob_crc_capture.txt",
    [string]$SourceDir = "E:\Projects\OutputBlaster"
)

$ErrorActionPreference = "Stop"

function Get-GameDir($profilePath) {
    [xml]$xml = Get-Content $profilePath
    $gp = $xml.GameProfile.GamePath
    if ([string]::IsNullOrEmpty($gp)) { return $null }
    return [System.IO.Path]::GetDirectoryName($gp)
}

function Deploy-Dll($gameDir) {
    if (!(Test-Path $gameDir)) { return $false }
    Copy-Item $DllPath "$gameDir\OutputBlaster.dll" -Force
    Write-Host "  Deployed to: $gameDir"
    $exeDir = Join-Path $gameDir "exe"
    if (Test-Path $exeDir) {
        Copy-Item $DllPath "$exeDir\OutputBlaster.dll" -Force
        Write-Host "  Deployed to: $exeDir"
    }
    return $true
}

function Run-Capture($profileName) {
    $profilePath = "$TpDir\UserProfiles\$profileName.xml"
    if (!(Test-Path $profilePath)) {
        Write-Host "  SKIP: Profile not found" -ForegroundColor DarkYellow
        return $null
    }

    Write-Host "`n=== Processing: $profileName ===" -ForegroundColor Cyan
    $gameDir = Get-GameDir $profilePath
    if (!$gameDir) {
        Write-Host "  SKIP: Empty GamePath" -ForegroundColor DarkYellow
        return $null
    }
    if (!(Test-Path $gameDir)) {
        Write-Host "  SKIP: Dir not found: $gameDir" -ForegroundColor DarkYellow
        return $null
    }

    Deploy-Dll $gameDir
    if (Test-Path $CrcFile) { Remove-Item $CrcFile -Force }

    $tpExe = "$TpDir\TeknoParrotUi.exe"
    Write-Host "  Launching: $tpExe --profile=$profileName.xml"
    $proc = Start-Process -FilePath $tpExe -ArgumentList "--profile=$profileName.xml" -PassThru -WindowStyle Hidden
    Write-Host "  PID: $($proc.Id)"
    Start-Sleep -Seconds 5

    $elapsed = 0
    $found = $false
    do {
        Start-Sleep -Seconds 2
        $elapsed += 2
        Write-Host "  ... waiting ($elapsed s)"
        if (Test-Path $CrcFile) {
            $crc = (Get-Content $CrcFile -Raw).Trim()
            if ($crc -match '^[0-9a-fA-F]{8}$') {
                Write-Host "  CRC captured: 0x$crc" -ForegroundColor Green
                $found = $true
                break
            }
        }
    } while ($elapsed -lt $TimeoutSeconds)

    Write-Host "  Closing processes..."
    @("TeknoParrotUi", "OpenParrotLoader", "sdaemon", "game", $profileName) | ForEach-Object {
        try { Get-Process -Name $_ -ErrorAction SilentlyContinue | Stop-Process -Force } catch {}
    }

    if ($found) { return "0x$crc" }
    Write-Host "  WARN: CRC not captured in ${TimeoutSeconds}s" -ForegroundColor Yellow
    return $null
}

function Update-DllMain($profileName, $crcHex) {
    $dllMain = "$SourceDir\dllmain.cpp"
    $content = Get-Content $dllMain -Raw
    $sanitized = $profileName -replace '[^a-zA-Z0-9]', ''

    if ($content -match [regex]::Escape($crcHex)) {
        Write-Host "  CRC $crcHex already in dllmain.cpp" -ForegroundColor Green
        return $true
    }

    $includeLine = '#include "Game Files/' + $sanitized + '.h"'
    if ($content -notmatch [regex]::Escape($includeLine)) {
        $idx = $content.LastIndexOf('#include "Game Files/')
        $eol = $content.IndexOf("`n", $idx)
        $content = $content.Substring(0, $eol + 1) + $includeLine + "`n" + $content.Substring($eol + 1)
    }

    $caseBlock = "`n`tcase ${crcHex}:  // ${profileName}`n`t`tgame = new ${sanitized};`n`t`tbreak;"
    $defIdx = $content.IndexOf("`tdefault:")
    if ($defIdx -ge 0) {
        $content = $content.Substring(0, $defIdx) + $caseBlock + $content.Substring($defIdx)
    }
    Set-Content $dllMain $content -NoNewline
    Write-Host "  Added CRC $crcHex to dllmain.cpp" -ForegroundColor Green

    $hFile = "$SourceDir\Game Files\$sanitized.h"
    $cppFile = "$SourceDir\Game Files\$sanitized.cpp"
    if (!(Test-Path $hFile) -or !(Test-Path $cppFile)) {
        Write-Host "  WARN: $sanitized.h/.cpp not found" -ForegroundColor Yellow
        return $false
    }
    return $true
}

function Rebuild-Dll {
    Write-Host "`n=== Rebuilding DLL ===" -ForegroundColor Cyan
    $msbuild = "C:\Program Files\Microsoft Visual Studio\18\Community\MSBuild\Current\Bin\MSBuild.exe"
    & $msbuild "$SourceDir\OutputBlaster.sln" /p:Configuration=Release /p:Platform=Win32 /p:PlatformToolset=v145 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) { Write-Host "  OK" -ForegroundColor Green; return $true }
    Write-Host "  FAILED" -ForegroundColor Red; return $false
}

# === MAIN ===
Write-Host "==========================================" -ForegroundColor Magenta
Write-Host "  OutputBlaster Auto CRC Capture" -ForegroundColor Magenta
Write-Host "==========================================" -ForegroundColor Magenta

if (!(Test-Path $DllPath)) { Write-Host "ERROR: DLL not found" -ForegroundColor Red; exit 1 }

$profiles = @()
if ($ProfileName -eq "all") {
    $profiles = Get-ChildItem "$TpDir\UserProfiles\*.xml" | ForEach-Object { $_.BaseName }
} elseif ($ProfileName -ne "") {
    $profiles = @($ProfileName)
} else {
    Write-Host 'Usage: .\auto-capture-crc.ps1 -ProfileName [name] (or -ProfileName all)' -ForegroundColor Yellow
    exit 1
}

$results = @{}
foreach ($p in $profiles) {
    $crc = Run-Capture $p
    $results[$p] = $crc
}

Write-Host "`n==========================================" -ForegroundColor Magenta
Write-Host "  RESULTS" -ForegroundColor Magenta
Write-Host "==========================================" -ForegroundColor Magenta
foreach ($p in $results.Keys) {
    $crc = $results[$p]
    if ($crc) { Write-Host "  $p => $crc" -ForegroundColor Green }
    else { Write-Host "  $p => NOT CAPTURED" -ForegroundColor Yellow }
}

$newGames = $results.GetEnumerator() | Where-Object { $_.Value -ne $null }
if ($newGames.Count -gt 0) {
    Write-Host "`nAuto-add to dllmain and rebuild? (y/N): " -ForegroundColor Cyan -NoNewline
    $key = [System.Console]::ReadKey($true)
    if ($key.KeyChar -eq 'y' -or $key.KeyChar -eq 'Y') {
        $anyMissing = $false
        $newGames | ForEach-Object {
            $ok = Update-DllMain $_.Key $_.Value
            if (!$ok) { $anyMissing = $true }
        }
        if (!$anyMissing) { Rebuild-Dll }
    }
}
