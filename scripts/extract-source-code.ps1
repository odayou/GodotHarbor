$OutputDir = Join-Path $PSScriptRoot "..\docs"
$OutputDir = Join-Path $OutputDir "source-code"
$ProjectDir = Join-Path $PSScriptRoot ".."
$SoftwareName = "GodotHarbor"
$Version = "v2.2.4"
$LinesPerPage = 50
$FrontPages = 30
$BackPages = 30

$ErrorActionPreference = "Stop"

$OutputDir = [System.IO.Path]::GetFullPath($OutputDir)
$ProjectDir = [System.IO.Path]::GetFullPath($ProjectDir)

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

$sourceDirs = @(
    (Join-Path $ProjectDir "src-tauri\src"),
    (Join-Path $ProjectDir "src")
)

$extensions = @("*.rs", "*.ts", "*.vue")

$allLines = [System.Collections.ArrayList]::new()

foreach ($dir in $sourceDirs) {
    if (-not (Test-Path $dir)) { continue }
    foreach ($ext in $extensions) {
        $files = Get-ChildItem -Path $dir -Recurse -Filter $ext -File
        foreach ($file in $files) {
            $relPath = $file.FullName.Substring($ProjectDir.Length + 1)
            $allLines.Add("// File: $relPath") | Out-Null
            $content = Get-Content $file.FullName -Encoding UTF8
            foreach ($line in $content) {
                $cleaned = $line
                $cleaned = $cleaned -creplace '[\w\.-]+@[\w\.-]+\.\w+', '<email>'
                $cleaned = $cleaned -creplace 'https?://(www\.)?[\w-]+\.[\w./-]+', '<url>'
                $cleaned = $cleaned -creplace 'authors\s*=\s*\[.*?\]', 'authors = ["<author>"]'
                $cleaned = $cleaned -creplace 'author\s*=\s*".*?"', 'author = "<author>"'
                $cleaned = $cleaned -creplace 'Author\s*:\s*.*', 'Author: <author>'
                $cleaned = $cleaned -creplace 'Maintainer\s*:\s*.*', 'Maintainer: <maintainer>'
                $allLines.Add($cleaned) | Out-Null
            }
            $allLines.Add("") | Out-Null
        }
    }
}

$totalLines = $allLines.Count
$totalPages = [math]::Ceiling($totalLines / $LinesPerPage)

Write-Host "Total code lines: $totalLines"
Write-Host "Total pages: $totalPages"

$frontEndLine = $FrontPages * $LinesPerPage
if ($frontEndLine -gt $totalLines) { $frontEndLine = $totalLines }

$backStartLine = $totalLines - ($BackPages * $LinesPerPage)
if ($backStartLine -lt $frontEndLine) { $backStartLine = $frontEndLine }

$headerText = "Software: $SoftwareName, Version: $Version"

$sbFront = [System.Text.StringBuilder]::new()
for ($p = 0; $p -lt $FrontPages; $p++) {
    $pageStart = $p * $LinesPerPage
    if ($pageStart -ge $frontEndLine) { break }
    $pageEnd = [math]::Min($pageStart + $LinesPerPage, $frontEndLine)
    $pageNum = $p + 1
    $divider = "=" * 80
    [void]$sbFront.AppendLine($divider)
    [void]$sbFront.AppendLine("| $headerText | Page $pageNum |")
    [void]$sbFront.AppendLine($divider)
    for ($i = $pageStart; $i -lt $pageEnd; $i++) {
        [void]$sbFront.AppendLine($allLines[$i])
    }
    [void]$sbFront.AppendLine("")
}

$frontFile = Join-Path $OutputDir "source-code-front-30-pages.md"
$sbFront.ToString() | Out-File -FilePath $frontFile -Encoding UTF8 -Force
Write-Host "Written: $frontFile"

$sbBack = [System.Text.StringBuilder]::new()
for ($p = 0; $p -lt $BackPages; $p++) {
    $pageStart = $backStartLine + ($p * $LinesPerPage)
    if ($pageStart -ge $totalLines) { break }
    $pageEnd = [math]::Min($pageStart + $LinesPerPage, $totalLines)
    $pageNum = $p + 1
    $divider = "=" * 80
    [void]$sbBack.AppendLine($divider)
    [void]$sbBack.AppendLine("| $headerText | Page $pageNum |")
    [void]$sbBack.AppendLine($divider)
    for ($i = $pageStart; $i -lt $pageEnd; $i++) {
        [void]$sbBack.AppendLine($allLines[$i])
    }
    [void]$sbBack.AppendLine("")
}

$backFile = Join-Path $OutputDir "source-code-back-30-pages.md"
$sbBack.ToString() | Out-File -FilePath $backFile -Encoding UTF8 -Force
Write-Host "Written: $backFile"

Write-Host "Done!"
