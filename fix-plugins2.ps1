$content = [System.IO.File]::ReadAllText('src/views/Plugins.vue', [System.Text.Encoding]::UTF8)

$lines = $content -split "`n"
$newLines = [System.Collections.ArrayList]::new()
$skipNext = $false
for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    if ($line -match 'binding\.mount_path\s*\}\}' -and $line -notmatch 'deletePluginBindings\.filter') {
        continue
    }
    if ($line -match '^\s*</div>\s*$' -and $i -gt 0 -and $lines[$i-1] -match 'binding\.mount_path' -and $lines[$i-1] -notmatch 'deletePluginBindings\.filter') {
        continue
    }
    [void]$newLines.Add($line)
}
$content = $newLines -join "`n"

[System.IO.File]::WriteAllText('src/views/Plugins.vue', $content, [System.Text.Encoding]::UTF8)
Write-Host "Done"
