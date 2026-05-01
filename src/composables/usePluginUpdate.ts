import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { useToast } from '@/composables/useToast'
import type { PluginUpdateInfo } from '@/types'

export function usePluginUpdate(options: {
  loadPlugins: (force?: boolean) => Promise<void>
}) {
  const toast = useToast()
  const { t } = useI18n()

  const showUpdatesDialog = ref(false)
  const pluginUpdates = ref<PluginUpdateInfo[]>([])
  const isCheckingUpdates = ref(false)
  const isBatchUpdating = ref(false)
  const expandedReleaseNotes = ref<Set<string>>(new Set())

  const updatablePluginIds = computed(() =>
    pluginUpdates.value.filter(u => u.update_available).map(u => u.plugin_id)
  )

  const checkPluginUpdates = async () => {
    isCheckingUpdates.value = true
    try {
      pluginUpdates.value = await api.checkPluginUpdates()
      showUpdatesDialog.value = true
    } catch (error) {
      toast.error(t('common.loadFailed', { error }))
    } finally {
      isCheckingUpdates.value = false
    }
  }

  const reapplyBindingsForPlugin = async (pluginId: string) => {
    try {
      const projects = await api.getProjects()
      const bindingResults = await Promise.allSettled(
        projects.map(p => api.getProjectBindings(p.project_id))
      )
      const projectIdsToApply: string[] = []
      bindingResults.forEach((result, i) => {
        if (result.status === 'fulfilled' && result.value.some(b => b.plugin_id === pluginId)) {
          projectIdsToApply.push(projects[i].project_id)
        }
      })
      await Promise.allSettled(
        projectIdsToApply.map(id => api.applyChanges(id))
      )
    } catch {
      // ignore reapply errors
    }
  }

  const updateGitPlugin = async (pluginId: string) => {
    try {
      const result = await api.updateGitPlugin(pluginId)
      toast.success(t('plugins.updateSuccess', { name: result.name }))
      await options.loadPlugins()
      await reapplyBindingsForPlugin(pluginId)
    } catch (error) {
      toast.error(t('common.loadFailed', { error }))
    }
  }

  const batchUpdatePlugins = async () => {
    isBatchUpdating.value = true
    let successCount = 0
    let failCount = 0
    const ids = [...updatablePluginIds.value]
    const concurrency = 3
    const chunks: string[][] = []
    for (let i = 0; i < ids.length; i += concurrency) {
      chunks.push(ids.slice(i, i + concurrency))
    }
    for (const chunk of chunks) {
      const results = await Promise.allSettled(chunk.map(id => api.updateGitPlugin(id)))
      for (const r of results) {
        if (r.status === 'fulfilled') successCount++
        else failCount++
      }
    }
    isBatchUpdating.value = false
    if (failCount > 0) {
      toast.warning(t('plugins.updateCheck.batchPartial', { success: successCount, failed: failCount }))
    } else {
      toast.success(t('plugins.updateCheck.batchSuccess', { count: successCount }))
    }
    await options.loadPlugins()
    pluginUpdates.value = await api.checkPluginUpdates()
  }

  return {
    showUpdatesDialog,
    pluginUpdates,
    isCheckingUpdates,
    isBatchUpdating,
    expandedReleaseNotes,
    updatablePluginIds,
    checkPluginUpdates,
    updateGitPlugin,
    reapplyBindingsForPlugin,
    batchUpdatePlugins,
  }
}
