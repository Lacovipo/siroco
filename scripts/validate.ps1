# validate.ps1 - Harness completo Siroco
# Perft + Autoplay + Bench + release perft speed
# Uso: powershell -ExecutionPolicy Bypass -File scripts/validate.ps1

$ErrorActionPreference = "Continue"
$start = Get-Date

Write-Host "=== Siroco Validation Harness ===" -ForegroundColor Cyan
Write-Host "Fecha: $start`n"

# 1. cargo test (unit perft)
Write-Host "[1/4] cargo test -- perft basico..." -ForegroundColor Yellow
cargo test -- --nocapture 2>&1 | Tee-Object -Variable testOut | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "FALLO perft basico" -ForegroundColor Red; exit 1 }
Write-Host "OK perft basico" -ForegroundColor Green

# 2. perft extendido
Write-Host "`n[2/4] perft_extendido..." -ForegroundColor Yellow
cargo test --test perft_extended -- --nocapture 2>&1 | Tee-Object -Variable extOut | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "FALLO perft extendido" -ForegroundColor Red; exit 1 }
Write-Host "OK perft extendido" -ForegroundColor Green

# 3. autoplay (no ilegales, hash consistente)
Write-Host "`n[3/4] autoplay (5 partidas depth 2-3, ~15s)..." -ForegroundColor Yellow
cargo test --test autoplay -- --nocapture 2>&1 | Tee-Object -Variable autoOut | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "FALLO autoplay" -ForegroundColor Red; exit 1 }
Write-Host "OK autoplay" -ForegroundColor Green

# 4. release build + perft speed + bench
Write-Host "`n[4/4] release build + perft 5 speed + bench..." -ForegroundColor Yellow
cargo build --release 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "FALLO build release" -ForegroundColor Red; exit 1 }

$perftResult = Write-Output "perft 5`nquit" | .\target\release\siroco.exe 2>&1 | Out-String
Write-Host $perftResult
if ($perftResult -notmatch "4865609") { Write-Host "FALLO perft 5 nodes" -ForegroundColor Red; exit 1 }
# extrae NPS
if ($perftResult -match "NPS:\s+(\d+)") { Write-Host "Perft NPS: $($Matches[1])" -ForegroundColor Gray }

$benchResult = Write-Output "bench 8`nquit" | .\target\release\siroco.exe 2>&1 | Out-String
Write-Host $benchResult

# UCI smoke
Write-Host "`n[extra] UCI smoke test..." -ForegroundColor Yellow
$uciResult = Write-Output "uci`nisready`nposition startpos`ngo depth 4`nquit" | .\target\release\siroco.exe 2>&1 | Out-String
if ($uciResult -notmatch "bestmove") { Write-Host "FALLO UCI smoke" -ForegroundColor Red; exit 1 }
Write-Host "OK UCI" -ForegroundColor Green

$elapsed = (Get-Date) - $start
Write-Host "`n=== VALIDACION COMPLETA OK ===" -ForegroundColor Green
Write-Host "Tiempo total: $($elapsed.TotalSeconds.ToString('0.0'))s"
Write-Host "Proximo paso: SPRT con cutechess-cli para validar parches (ver scripts/sprt.py)"
