import { ref } from 'vue'
import { api } from '@/api'
import type { Project, BatchBindingRequest } from '@/types'
import { useToast } from '@/composables/useToast'
import { useI18n } from 'vue-i18n'

export type AutoSetupStep = 'idle' | 'scanning-plugins' | 'importing-plugins' | 'binding-plugins' | 'applying-changes' | 'discovering-engines' | 'done'

export function useAutoSetup() {
  const toast = useToast()
  const { t } = useI18n()

  const isRunning = ref(false)
  const currentStep = ref<AutoSetupStep>('idle')
  const stepMessage = ref('')
  const lastResult = ref<{
    pluginsImported: number
    bindingsCreated: number
    enginesDiscovered: number
    projectsAffected: string[]
  } | null>(null)

  const runAutoSetup = async (targetProjects?: Project[]) => {
    if (isRunning.value) return
    isRunning.value = true
    lastResult.value = null

    let pluginsImported = 0
    let bindingsCreated = 0
    let enginesDiscovered = 0
    const projectsAffected: string[] = []

    try {
      currentStep.value = 'scanning-plugins'
      stepMessage.value = t('autoSetup.scanningPlugins')
      const scannedPlugins = await api.scanProjectPlugins()

      if (scannedPlugins.length === 0) {
        currentStep.value = 'done'
        stepMessage.value = t('autoSetup.noPluginsFound')
        lastResult.value = { pluginsImported: 0, bindingsCreated: 0, enginesDiscovered: 0, projectsAffected: [] }
        return
      }

      currentStep.value = 'importing-plugins'
      stepMessage.value = t('autoSetup.importingPlugins', { count: scannedPlugins.length })
      const importedPlugins = await api.importPluginsFromProjects('copy')
      pluginsImported = importedPlugins.length

      if (importedPlugins.length > 0) {
        currentStep.value = 'binding-plugins'
        stepMessage.value = t('autoSetup.bindingPlugins')

        const allProjects = targetProjects || await api.getProjects()
        const bindings: BatchBindingRequest[] = []

        for (const project of allProjects) {
          const projectScanned = scannedPlugins.filter(sp => sp.path.startsWith(project.path))
          if (projectScanned.length === 0) continue

          const existingBindings = await api.getProjectBindings(project.project_id)
          const existingPluginIds = new Set(existingBindings.map(b => b.plugin_id))

          for (const scanned of projectScanned) {
            const matchedPlugin = importedPlugins.find(ip => {
              const pluginDirName = scanned.path.split('/').pop()?.toLowerCase() || ''
              return ip.name.toLowerCase() === scanned.plugin_name.toLowerCase() ||
                ip.source.url.toLowerCase().includes(pluginDirName)
            })

            if (!matchedPlugin || existingPluginIds.has(matchedPlugin.plugin_id)) continue

            const version = matchedPlugin.versions[0]
            if (!version) continue

            const unit = version.units[0]
            if (!unit) continue

            const addonsPrefix = project.path.replace(/\\/g, '/') + '/addons/'
            let mountPath = scanned.path.replace(/\\/g, '/').replace(addonsPrefix, '')
            if (!mountPath || mountPath === scanned.path.replace(/\\/g, '/')) {
              mountPath = unit.subdirectory || scanned.plugin_name
            }

            bindings.push({
              project_id: project.project_id,
              plugin_id: matchedPlugin.plugin_id,
              version_id: version.version_id,
              unit_id: unit.unit_id,
              mount_path: mountPath,
              subdirectory: unit.subdirectory || ''
            })

            if (!projectsAffected.includes(project.name)) {
              projectsAffected.push(project.name)
            }
          }
        }

        if (bindings.length > 0) {
          const bindResult = await api.batchBindPlugins(bindings)
          bindingsCreated = bindResult.success_count

          currentStep.value = 'applying-changes'
          stepMessage.value = t('autoSetup.applyingChanges')
          const affectedProjectIds = [...new Set(bindings.map(b => b.project_id))]
          await api.batchApplyChanges(affectedProjectIds)
        }
      }

      currentStep.value = 'discovering-engines'
      stepMessage.value = t('autoSetup.discoveringEngines')
      try {
        const settings = await api.getSettings()
        if (settings.auto_discover_engines) {
          const newEngines = await api.autoDiscoverEngines()
          enginesDiscovered = newEngines.length

          if (newEngines.length > 0) {
            const allProjects2 = targetProjects || await api.getProjects()
            const allEngines = await api.getEngines()
            for (const project of allProjects2) {
              const existingBinding = await api.getProjectEngineBinding(project.project_id)
              if (existingBinding) continue

              const matchedEngine = findMatchingEngine(project, allEngines)
              if (matchedEngine) {
                try {
                  await api.bindProjectEngine(project.project_id, matchedEngine.engine_id, '')
                } catch {}
              }
            }
          }
        }
      } catch {}

      currentStep.value = 'done'
      stepMessage.value = t('autoSetup.complete', {
        plugins: pluginsImported,
        bindings: bindingsCreated,
        engines: enginesDiscovered
      })

      lastResult.value = { pluginsImported, bindingsCreated, enginesDiscovered, projectsAffected }

      if (pluginsImported > 0 || bindingsCreated > 0 || enginesDiscovered > 0) {
        toast.success(stepMessage.value)
      } else {
        toast.info(t('autoSetup.nothingToDo'))
      }
    } catch (error: any) {
      console.error('Auto setup failed:', error)
      toast.error(t('autoSetup.failed', { error: error?.message || error }))
      currentStep.value = 'idle'
    } finally {
      isRunning.value = false
    }
  }

  return {
    isRunning,
    currentStep,
    stepMessage,
    lastResult,
    runAutoSetup,
  }
}

function findMatchingEngine(project: Project, engines: { engine_id: string; engine_type: string; version: string; is_default: boolean }[]) {
  const defaultEngine = engines.find(e => e.is_default)
  const projectMajor = project.godot_version?.split('.')[0]

  const versionMatch = engines.find(e => {
    const engineMajor = e.version?.split('.')[0]
    return engineMajor && projectMajor && engineMajor === projectMajor
  })

  const typeMatch = engines.find(e => {
    if (projectMajor === '4') return e.engine_type === 'Godot4'
    if (projectMajor === '3') return e.engine_type === 'Godot3'
    return false
  })

  return typeMatch || versionMatch || defaultEngine || engines[0]
}
