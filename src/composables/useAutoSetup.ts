import { ref, computed } from 'vue'
import { api } from '@/api'
import type { Project, Plugin, BatchBindingRequest, ScannedPlugin } from '@/types'
import { useToast } from '@/composables/useToast'
import { useI18n } from 'vue-i18n'
import { emit } from '@tauri-apps/api/event'

export type AutoSetupStep = 'idle' | 'scanning-projects' | 'scanning-plugins' | 'importing-plugins' | 'binding-plugins' | 'applying-changes' | 'done'

const SETUP_STEPS: AutoSetupStep[] = ['scanning-projects', 'scanning-plugins', 'importing-plugins', 'binding-plugins', 'applying-changes']

function normalizePath(p: string): string {
  return p.replace(/\\/g, '/').toLowerCase()
}

function getPluginDirName(pluginPath: string): string {
  const segments = normalizePath(pluginPath).split('/')
  return segments[segments.length - 1] || ''
}

const isRunning = ref(false)
const currentStep = ref<AutoSetupStep>('idle')
const stepMessage = ref('')
let doneTimer: ReturnType<typeof setTimeout> | null = null
const lastResult = ref<{
  projectsScanned: number
  pluginsImported: number
  bindingsCreated: number
  projectsAffected: string[]
} | null>(null)

const stepIndex = computed(() => {
  const idx = SETUP_STEPS.indexOf(currentStep.value)
  return idx >= 0 ? idx : -1
})

const progressPercent = computed(() => {
  if (currentStep.value === 'idle') return 0
  if (currentStep.value === 'done') return 100
  if (stepIndex.value < 0) return 0
  return Math.round(((stepIndex.value + 1) / SETUP_STEPS.length) * 100)
})

function setStep(step: AutoSetupStep, message: string) {
  currentStep.value = step
  stepMessage.value = message
}

async function buildAutoBindings(
  allProjects: Project[],
  allPlugins: Plugin[],
  scannedPlugins: ScannedPlugin[]
): Promise<{ bindings: BatchBindingRequest[]; projectsAffected: string[] }> {
  const allBindingsMap = new Map<string, Set<string>>()
  const bindingResults = await Promise.allSettled(
    allProjects.map(p => api.getProjectBindings(p.project_id))
  )
  bindingResults.forEach((result, i) => {
    const existingIds = new Set<string>()
    if (result.status === 'fulfilled') {
      result.value.forEach(b => existingIds.add(b.plugin_id))
    }
    allBindingsMap.set(allProjects[i].project_id, existingIds)
  })

  const projectMap = new Map(allProjects.map(p => [p.project_id, p]))

  const bindings: BatchBindingRequest[] = []
  const projectsAffected: string[] = []

  for (const scanned of scannedPlugins) {
    const project = scanned.project_id
      ? projectMap.get(scanned.project_id)
      : allProjects.find(p => {
          const normProjectPath = normalizePath(p.path)
          const normScannedPath = normalizePath(scanned.path)
          return normScannedPath.startsWith(normProjectPath + '/addons/') ||
                 normScannedPath.startsWith(normProjectPath + '\\addons\\')
        })

    if (!project) continue

    const existingPluginIds = allBindingsMap.get(project.project_id) || new Set<string>()

    const matchedPlugin = allPlugins.find(ip => {
      if (ip.name.toLowerCase() === scanned.plugin_name.toLowerCase()) return true
      const dirName = getPluginDirName(scanned.path)
      if (dirName && normalizePath(ip.source.url).includes(dirName)) return true
      const normPluginPath = normalizePath(scanned.path)
      const normSourceUrl = normalizePath(ip.source.url)
      if (normSourceUrl && normPluginPath.includes(normSourceUrl)) return true
      return false
    })

    if (!matchedPlugin || existingPluginIds.has(matchedPlugin.plugin_id)) continue

    const version = matchedPlugin.versions[0]
    if (!version) continue
    const unit = version.units[0]
    if (!unit) continue

    const normScannedPath = normalizePath(scanned.path)
    const addonsIdx = normScannedPath.indexOf('/addons/')
    const isAssetPack = matchedPlugin.asset_type === 'AssetPack'
    const mountPath = addonsIdx !== -1
      ? normScannedPath.substring(addonsIdx + '/addons/'.length)
      : isAssetPack ? `assets/${unit.dir_name && unit.dir_name !== 'payload' ? unit.dir_name : (unit.subdirectory ? unit.subdirectory.replace(/\\/g, '/').split('/').pop() : matchedPlugin.name)}` : `addons/${unit.dir_name && unit.dir_name !== 'payload' ? unit.dir_name : (unit.subdirectory ? unit.subdirectory.replace(/\\/g, '/').split('/').pop() : matchedPlugin.name)}`

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

  return { bindings, projectsAffected }
}

async function scanAndImportPlugins(t: ReturnType<typeof useI18n>['t'], allProjects: Project[]): Promise<{ pluginsImported: number; bindingsCreated: number; projectsAffected: string[] }> {
  setStep('scanning-plugins', t('autoSetup.scanningPlugins'))
  const scannedPlugins = await api.scanProjectPlugins()

  if (scannedPlugins.length === 0) {
    return { pluginsImported: 0, bindingsCreated: 0, projectsAffected: [] }
  }

  setStep('importing-plugins', t('autoSetup.importingPlugins', { count: scannedPlugins.length }))
  const importedPlugins = await api.importPluginsFromProjects()

  setStep('binding-plugins', t('autoSetup.bindingPlugins'))
  const allPlugins = await api.getPlugins()

  const { bindings, projectsAffected } = await buildAutoBindings(allProjects, allPlugins, scannedPlugins)

  let bindingsCreated = 0
  if (bindings.length > 0) {
    const bindResult = await api.batchBindPlugins(bindings)
    bindingsCreated = bindResult.success_count

    setStep('applying-changes', t('autoSetup.applyingChanges'))
    const affectedProjectIds = [...new Set(bindings.map(b => b.project_id))]
    await api.batchApplyChanges(affectedProjectIds)
  }

  return { pluginsImported: importedPlugins.length, bindingsCreated, projectsAffected }
}

export function useAutoSetup() {
  const toast = useToast()
  const { t } = useI18n()
  const runAutoSetup = async (targetProjects?: Project[], skipProjectScan = false, checkCompletionMark = false) => {
    if (isRunning.value) return

    if (checkCompletionMark) {
      try {
        const needed = await api.checkAutoSetupNeeded()
        if (!needed) {
          return
        }
      } catch {
        // check failed, proceed with setup
      }
    }

    isRunning.value = true
    lastResult.value = null

    let projectsScanned = 0

    try {
      const settings = await api.getSettings()

      const existingProjects = targetProjects || await api.getProjects()

      if (!skipProjectScan && settings.auto_scan_on_startup) {
        let scanDirs = settings.scan_directories
        if (scanDirs.length === 0) {
          scanDirs = await api.getDefaultScanDirs()
        }
        if (scanDirs.length > 0) {
          setStep('scanning-projects', t('autoSetup.scanningProjects'))
          const newProjects = await api.scanProjects(scanDirs)
          projectsScanned = newProjects.length
        }
      }

      const allProjects = existingProjects.length > 0 ? existingProjects : await api.getProjects()

      if (allProjects.length === 0) {
        await api.markAutoSetupDone()
        setStep('done', t('autoSetup.complete'))
        lastResult.value = {
          projectsScanned,
          pluginsImported: 0,
          bindingsCreated: 0,
          projectsAffected: []
        }
        isRunning.value = false
        return
      }

      const pluginResult = await scanAndImportPlugins(t, allProjects)

      await api.markAutoSetupDone()

      setStep('done', t('autoSetup.complete', {
        projects: projectsScanned,
        plugins: pluginResult.pluginsImported,
        bindings: pluginResult.bindingsCreated,
      }))

      lastResult.value = {
        projectsScanned,
        pluginsImported: pluginResult.pluginsImported,
        bindingsCreated: pluginResult.bindingsCreated,
        projectsAffected: pluginResult.projectsAffected
      }

      if (projectsScanned > 0 || pluginResult.pluginsImported > 0 || pluginResult.bindingsCreated > 0) {
        toast.success(stepMessage.value)
      }

      if (doneTimer) clearTimeout(doneTimer)
      doneTimer = setTimeout(() => {
        if (currentStep.value === 'done') {
          currentStep.value = 'idle'
        }
        doneTimer = null
      }, 5000)

      emit('auto-setup-complete')
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
    progressPercent,
    stepIndex,
    stepTotal: SETUP_STEPS.length,
    runAutoSetup,
  }
}
