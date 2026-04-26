$content = [System.IO.File]::ReadAllText('src/views/Plugins.vue', [System.Text.Encoding]::UTF8)

$old1 = "deletePluginBindings.length, name: deletePluginName"
$new1 = "new Set(deletePluginBindings.map(b => b.project_id)).size, name: deletePluginName"
$content = $content.Replace($old1, $new1)

$old2 = '<div class="space-y-1 max-h-32 overflow-y-auto">'
$new2 = '<div class="space-y-2 max-h-40 overflow-y-auto">'
$content = $content.Replace($old2, $new2)

$old3 = '<div v-for="binding in deletePluginBindings" :key="binding.project_id + binding.mount_path" class="text-xs text-red-600 dark:text-red-400">'
$new3 = '<div v-for="projectId in [...new Set(deletePluginBindings.map(b => b.project_id))]" :key="projectId" class="text-xs"><div class="font-medium text-red-700 dark:text-red-400">{{ deletePluginProjects.get(projectId) || projectId }}</div><div v-for="binding in deletePluginBindings.filter(b => b.project_id === projectId)" :key="binding.mount_path" class="text-red-600 dark:text-red-400 pl-3">{{ binding.mount_path }}</div></div>'
$content = $content.Replace($old3, $new3)

[System.IO.File]::WriteAllText('src/views/Plugins.vue', $content, [System.Text.Encoding]::UTF8)
Write-Host "Done"
