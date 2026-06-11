import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { EngineModulesInfo, ModuleType, ModuleInstallProgress } from '@/types'
import { useToast } from '@/composables/useToast'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export function useEngineModules(engineId?: string) {
  const { t } = useI18n()
  const toast = useToast()
  const modulesInfo = ref<EngineModulesInfo | null>(null)
  const allModulesInfo = ref<EngineModulesInfo[]>([])
  const isLoading = ref(false)
  const installingModules = ref<Set<string>>(new Set())
  const installProgress = ref<ModuleInstallProgress | null>(null)

  let unlistenProgress: UnlistenFn | null = null

  onMounted(async () => {
    unlistenProgress = await listen('module-install-progress', (event) => {
      const progress = event.payload as ModuleInstallProgress
      installProgress.value = progress
      if (progress.stage === 'complete') {
        const key = `${progress.module_type}_${progress.version}`
        installingModules.value = new Set([...installingModules.value].filter(k => k !== key))
        // Refresh modules after installation
        if (engineId) {
          loadModules(engineId)
        }
      } else if (progress.stage === 'failed') {
        const key = `${progress.module_type}_${progress.version}`
        installingModules.value = new Set([...installingModules.value].filter(k => k !== key))
      }
    })
  })

  onUnmounted(() => {
    if (unlistenProgress) {
      unlistenProgress()
    }
  })

  const loadModules = async (id: string) => {
    isLoading.value = true
    try {
      modulesInfo.value = await api.getEngineModules(id)
    } catch (error) {
      toast.error(t('common.loadFailed', { error: String(error) }))
    } finally {
      isLoading.value = false
    }
  }

  const loadAllModules = async () => {
    isLoading.value = true
    try {
      allModulesInfo.value = await api.getAllEnginesModules()
    } catch (error) {
      toast.error(t('common.loadFailed', { error: String(error) }))
    } finally {
      isLoading.value = false
    }
  }

  const installModule = async (id: string, moduleType: ModuleType) => {
    const key = `${moduleType}_${id}`
    installingModules.value = new Set([...installingModules.value, key])
    try {
      await api.installEngineModule(id, moduleType)
      toast.success(t('engines.modules.installSuccess', { type: moduleType }))
    } catch (error) {
      installingModules.value = new Set([...installingModules.value].filter(k => k !== key))
      toast.error(t('engines.modules.installFailed', { error: String(error) }))
    }
  }

  const refreshModules = async (id?: string) => {
    if (id) {
      await loadModules(id)
    } else if (engineId) {
      await loadModules(engineId)
    }
  }

  const getModuleTypeLabel = (type: ModuleType): string => {
    switch (type) {
      case 'DotNet': return '.NET'
      case 'Android': return 'Android'
      case 'IOS': return 'iOS'
      case 'Web': return 'Web'
      case 'Linux': return 'Linux'
      case 'Windows': return 'Windows'
      case 'MacOS': return 'macOS'
      case 'Editor': return t('engines.modules.editor')
      default: return type
    }
  }

  const getModuleTypeIcon = (type: ModuleType): string => {
    switch (type) {
      case 'DotNet': return 'dotnet'
      case 'Android': return 'android'
      case 'IOS': return 'ios'
      case 'Web': return 'web'
      case 'Linux': return 'linux'
      case 'Windows': return 'windows'
      case 'MacOS': return 'macos'
      case 'Editor': return 'editor'
      default: return 'module'
    }
  }

  const isModuleInstalling = (moduleType: ModuleType): boolean => {
    return installingModules.value.has(`${moduleType}_${engineId || ''}`)
  }

  return {
    modulesInfo,
    allModulesInfo,
    isLoading,
    installingModules,
    installProgress,
    loadModules,
    loadAllModules,
    installModule,
    refreshModules,
    getModuleTypeLabel,
    getModuleTypeIcon,
    isModuleInstalling,
  }
}

export function useProjectMissingModules() {
  const { t } = useI18n()

  const missingModules = ref<ModuleType[]>([])
  const isLoading = ref(false)

  const checkMissing = async (projectId: string) => {
    isLoading.value = true
    try {
      missingModules.value = await api.checkProjectMissingModules(projectId)
    } catch (error) {
      // Silently fail - missing modules check is informational
      missingModules.value = []
    } finally {
      isLoading.value = false
    }
  }

  const getModuleTypeLabel = (type: ModuleType): string => {
    switch (type) {
      case 'DotNet': return '.NET'
      case 'Android': return 'Android'
      case 'IOS': return 'iOS'
      case 'Web': return 'Web'
      case 'Linux': return 'Linux'
      case 'Windows': return 'Windows'
      case 'MacOS': return 'macOS'
      case 'Editor': return t('engines.modules.editor')
      default: return type
    }
  }

  return {
    missingModules,
    isLoading,
    checkMissing,
    getModuleTypeLabel,
  }
}
