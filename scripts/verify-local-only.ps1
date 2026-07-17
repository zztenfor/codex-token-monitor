$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$quotaClient = Join-Path $projectRoot "src-tauri\src\services\quota\usage_api_client.rs"
$patterns = @('fetch("http', "fetch('http", "axios", "XMLHttpRequest", "reqwest", "hyper::Client")
$files = Get-ChildItem $projectRoot\src, $projectRoot\src-tauri\src -Recurse -File
$matches = $files | Select-String -SimpleMatch -Pattern $patterns
$unexpected = $matches | Where-Object { $_.Path -ne $quotaClient }
if ($unexpected) {
  $unexpected | Format-Table Path, LineNumber, Line
  throw "Unexpected network client usage detected."
}

$clientSource = Get-Content -LiteralPath $quotaClient -Raw
if (-not $clientSource.Contains('https://chatgpt.com/backend-api/wham/usage')) {
  throw "The quota client endpoint is not the approved Codex usage endpoint."
}
if (-not $clientSource.Contains('.get(&self.endpoint)') -or $clientSource.Contains('.post(')) {
  throw "The quota client must remain GET-only."
}

Write-Host "Network boundary check passed: only the fixed Codex quota GET is enabled."
