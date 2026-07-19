$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$quotaClient = Join-Path $projectRoot "src-tauri\src\services\quota\usage_api_client.rs"
$pricingClient = Join-Path $projectRoot "src-tauri\src\commands\mod.rs"
$patterns = @('fetch("http', "fetch('http", "axios", "XMLHttpRequest", "reqwest", "hyper::Client")
$files = Get-ChildItem $projectRoot\src, $projectRoot\src-tauri\src -Recurse -File
$matches = $files | Select-String -SimpleMatch -Pattern $patterns
$unexpected = $matches | Where-Object { $_.Path -notin @($quotaClient, $pricingClient) }
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

$pricingSource = Get-Content -LiteralPath $pricingClient -Raw
if (-not $pricingSource.Contains('https://developers.openai.com/api/docs/pricing')) {
  throw "The pricing endpoint is not the approved public OpenAI pricing page."
}
if (-not $pricingSource.Contains('.get(PRICING_SOURCE_URL)') -or $pricingSource.Contains('.post(PRICING_SOURCE_URL)')) {
  throw "The pricing client must remain GET-only."
}

Write-Host "Network boundary check passed: only the fixed Codex quota GET and public pricing GET are enabled."
