import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { Project, MatchedEngine, Engine } from '@/types'
import { useToast } from '@/composables/useToast'

export function useEngineLauncher(onLaunched?: () => void) {
  const { t } = useI18n()
  const toast = useToast()

  const showEngineSelectDialog = ref(false)
  const engineSelectProject = ref<Project | null>(null)
  const matchedEngines = ref<MatchedEngine[]>([])
  const isLoadingEngines = ref(false)
  const engineSelectMode = ref<'launch' | 'select'>('launch')

  const openProjectWithEngine = async (project: Project, engines?: Engine[]) => {
    if (project.last_used_engine_id) {
      try {
        const engineList = engines || await api.getEngines()
        const engineExists = engineList.some(e => e.engine_id === project.last_used_engine_id)
        if (engineExists) {
          await api.launchEngine(project.last_used_engine_id!, project.path, project.project_id)
          toast.success(t('engines.launchSuccess'))
          onLaunched?.()
          return
        }
      } catch (error) {
        toast.error(t('projects.launchFailed', { error: String(error) }))
        return
      }
    }
    isLoadingEngines.value = true
    showEngineSelectDialog.value = true
    engineSelectProject.value = project
    engineSelectMode.value = 'launch'
    try {
      const result = await api.findMatchingEngines(project.godot_version)
      matchedEngines.value = result
    } catch (error) {
      toast.error(t('projects.launchFailed', { error }))
      matchedEngines.value = []
    } finally {
      isLoadingEngines.value = false
    }
  }

  const selectDefaultEngine = async (project: Project) => {
    isLoadingEngines.value = true
    showEngineSelectDialog.value = true
    engineSelectProject.value = project
    engineSelectMode.value = 'select'
    try {
      const result = await api.findMatchingEngines(project.godot_version)
      matchedEngines.value = result
    } catch (error) {
      toast.error(t('projects.launchFailed', { error }))
      matchedEngines.value = []
    } finally {
      isLoadingEngines.value = false
    }
  }

  const launchWithEngine = async (engineId: string) => {
    if (!engineSelectProject.value) return
    if (engineSelectMode.value === 'select') {
      try {
        await api.setProjectDefaultEngine(engineSelectProject.value.project_id, engineId)
        toast.success(t('projects.defaultEngineSet'))
        showEngineSelectDialog.value = false
        engineSelectProject.value = null
        onLaunched?.()
      } catch (error) {
        toast.error(t('projects.launchFailed', { error: String(error) }))
      }
      return
    }
    try {
      await api.launchEngine(engineId, engineSelectProject.value.path, engineSelectProject.value.project_id)
      toast.success(t('engines.launchSuccess'))
      showEngineSelectDialog.value = false
      engineSelectProject.value = null
      onLaunched?.()
    } catch (error) {
      toast.error(t('projects.launchFailed', { error: String(error) }))
    }
  }

  const closeEngineSelectDialog = () => {
    showEngineSelectDialog.value = false
    engineSelectProject.value = null
  }

  const getMatchLevelClass = (level: string) => {
    switch (level) {
      case 'exact': return 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400'
      case 'minor': return 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400'
      case 'major': return 'bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400'
      default: return ''
    }
  }

  const getMatchLevelLabel = (level: string) => {
    switch (level) {
      case 'exact': return t('projects.matchExact')
      case 'minor': return t('projects.matchMinor')
      case 'major': return t('projects.matchMajor')
      default: return level
    }
  }

  const getMatchLevelDesc = (level: string) => {
    switch (level) {
      case 'exact': return t('projects.matchExactDesc')
      case 'minor': return t('projects.matchMinorDesc')
      case 'major': return t('projects.matchMajorDesc')
      default: return ''
    }
  }

  return {
    showEngineSelectDialog,
    engineSelectProject,
    matchedEngines,
    isLoadingEngines,
    engineSelectMode,
    openProjectWithEngine,
    selectDefaultEngine,
    launchWithEngine,
    closeEngineSelectDialog,
    getMatchLevelClass,
    getMatchLevelLabel,
    getMatchLevelDesc,
  }
}
