<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import { api } from '@/api'
import { useOnboarding } from '@/composables/useOnboarding'
import { useLanguageDialog } from '@/composables/useLanguageDialog'
import { useAutoSetup } from '@/composables/useAutoSetup'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import type { Project, ScannedPlugin, Plugin } from '@/types'

const { t } = useI18n()

const currentStep = ref(0)
const { isVisible, hideOnboarding } = useOnboarding()
const { isVisible: languageDialogVisible } = useLanguageDialog()
const { runAutoSetup } = useAutoSetup()

const isScanning = ref(false)
const scannedProjects = ref<Project[]>([])
const selectedProjectIds = ref<Set<string>>(new Set())
const scannedPlugins = ref<ScannedPlugin[]>([])
const selectedPluginPaths = ref<Set<string>>(new Set())
const isImporting = ref(false)
const importedPlugins = ref<Plugin[]>([])
const isApplying = ref(false)
const applyResult = ref<any>(null)
const errorMessage = ref('')

const bindingMap = reactive<Map<string, Set<string>>>(new Map())

const checkFirstTime = async () => {
  try {
    const settings = await api.getSettings()
    if (settings.onboarding_completed) return
    const [projects, plugins] = await Promise.all([
      api.getProjects(),
      api.getPlugins()
    ])
    if (projects.length === 0 && plugins.length === 0) {
      if (languageDialogVisible.value) return
      isVisible.value = true
    }
  } catch {}
}

if (!languageDialogVisible.value) {
  checkFirstTime()
}

watch(languageDialogVisible, (visible) => {
  if (!visible) {
    checkFirstTime()
  }
})

const steps = computed(() => [
  { title: t('onboarding.welcome.title'), icon: 'welcome' },
  { title: t('onboarding.interactive.selectProject'), icon: 'project' },
  { title: t('onboarding.interactive.importPlugin'), icon: 'plugin' },
  { title: t('onboarding.interactive.configureBindings'), icon: 'bind' },
  { title: t('onboarding.interactive.bindAndApply'), icon: 'apply' },
])

const isLastStep = computed(() => currentStep.value === steps.value.length - 1)
const progress = computed(() => ((currentStep.value + 1) / steps.value.length) * 100)

const markOnboardingCompleted = async () => {
  try {
    const settings = await api.getSettings()
    settings.onboarding_completed = true
    await api.saveSettings(settings)
  } catch {}
}

const doScanProjects = async () => {
  isScanning.value = true
  errorMessage.value = ''
  try {
    const settings = await api.getSettings()
    const dirs = settings.scan_directories.length > 0
      ? settings.scan_directories
      : await api.getDefaultScanDirs()
    scannedProjects.value = await api.scanProjects(dirs)
    if (scannedProjects.value.length === 0) {
      errorMessage.value = t('onboarding.interactive.noProjectsFound')
    }
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    isScanning.value = false
  }
}

const doSelectProjectDir = async () => {
  try {
    const selected = await open({ directory: true, multiple: false, title: t('onboarding.interactive.selectProjectDir') })
    if (selected && typeof selected === 'string') {
      isScanning.value = true
      errorMessage.value = ''
      try {
        const project = await api.addProject(selected)
        if (!scannedProjects.value.find(p => p.project_id === project.project_id)) {
          scannedProjects.value.push(project)
        }
        selectedProjectIds.value.add(project.project_id)
      } catch (error) {
        errorMessage.value = String(error)
      } finally {
        isScanning.value = false
      }
    }
  } catch {}
}

const toggleProjectSelection = (projectId: string) => {
  const newSet = new Set(selectedProjectIds.value)
  if (newSet.has(projectId)) {
    newSet.delete(projectId)
  } else {
    newSet.add(projectId)
  }
  selectedProjectIds.value = newSet
}

const doScanPlugins = async () => {
  isScanning.value = true
  errorMessage.value = ''
  try {
    scannedPlugins.value = await api.scanProjectPlugins()
    if (scannedPlugins.value.length === 0) {
      errorMessage.value = t('onboarding.interactive.noPluginsFound')
    }
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    isScanning.value = false
  }
}

const togglePluginSelection = (path: string) => {
  if (selectedPluginPaths.value.has(path)) {
    selectedPluginPaths.value.delete(path)
  } else {
    selectedPluginPaths.value.add(path)
  }
}

const doImportPlugins = async () => {
  if (selectedPluginPaths.value.size === 0) return
  isImporting.value = true
  errorMessage.value = ''
  try {
    const result = await api.importPluginsFromProjects('reference')
    importedPlugins.value = result
    initBindingMap()
    currentStep.value = 3
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    isImporting.value = false
  }
}

const initBindingMap = () => {
  bindingMap.clear()
  for (const projectId of selectedProjectIds.value) {
    bindingMap.set(projectId, new Set(importedPlugins.value.map(p => p.plugin_id)))
  }
}

const toggleBinding = (projectId: string, pluginId: string) => {
  let set = bindingMap.get(projectId)
  if (!set) {
    set = new Set()
    bindingMap.set(projectId, set)
  }
  if (set.has(pluginId)) {
    set.delete(pluginId)
  } else {
    set.add(pluginId)
  }
}

const isPluginBoundToProject = (projectId: string, pluginId: string) => {
  return bindingMap.get(projectId)?.has(pluginId) ?? false
}

const selectAllForProject = (projectId: string) => {
  bindingMap.set(projectId, new Set(importedPlugins.value.map(p => p.plugin_id)))
}

const deselectAllForProject = (projectId: string) => {
  bindingMap.set(projectId, new Set())
}

const getBindingCount = (projectId: string) => {
  return bindingMap.get(projectId)?.size ?? 0
}

const totalBindings = computed(() => {
  let count = 0
  for (const set of bindingMap.values()) {
    count += set.size
  }
  return count
})

const doBindAndApply = async () => {
  if (totalBindings.value === 0) return
  isApplying.value = true
  errorMessage.value = ''
  try {
    for (const [projectId, pluginIds] of bindingMap.entries()) {
      for (const pluginId of pluginIds) {
        const plugin = importedPlugins.value.find(p => p.plugin_id === pluginId)
        if (!plugin) continue
        const version = plugin.versions[0]
        const unit = version?.units[0]
        if (!version || !unit) continue
        const mountPath = `addons/${unit.name}`
        await api.bindPlugin(projectId, plugin.plugin_id, version.version_id, unit.unit_id, mountPath, unit.subdirectory || '')
      }
    }
    const projectIds = Array.from(bindingMap.keys())
    for (const projectId of projectIds) {
      if ((bindingMap.get(projectId)?.size ?? 0) > 0) {
        applyResult.value = await api.applyChanges(projectId)
      }
    }
  } catch (error) {
    errorMessage.value = String(error)
  } finally {
    isApplying.value = false
  }
}

const next = () => {
  if (currentStep.value === 1 && selectedProjectIds.value.size > 0) {
    doScanPlugins()
    currentStep.value = 2
  } else if (currentStep.value === 1 && selectedProjectIds.value.size === 0) {
    finish()
  } else if (isLastStep) {
    finish()
  } else {
    currentStep.value++
  }
}

const skip = () => {
  finish()
}

const finish = async () => {
  hideOnboarding()
  currentStep.value = 0
  await markOnboardingCompleted()
  setTimeout(() => {
    runAutoSetup(undefined, false)
  }, 500)
}
</script>

<template>
  <div v-if="isVisible" class="fixed inset-0 bg-black/60 flex items-center justify-center z-[100] p-4">
    <div class="bg-white dark:bg-surface-card rounded-2xl shadow-2xl w-full max-w-2xl overflow-hidden">
      <div class="bg-primary-600 h-1.5">
        <div
          class="bg-primary-400 h-full transition-all duration-300"
          :style="{ width: progress + '%' }"
        />
      </div>

      <div class="p-8">
        <!-- Step 0: Welcome -->
        <template v-if="currentStep === 0">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center">
              <svg class="w-10 h-10 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.welcome.title') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center whitespace-pre-line mb-8">{{ t('onboarding.welcome.desc') }}</p>
        </template>

        <!-- Step 1: Select Projects (multi-select) -->
        <template v-if="currentStep === 1">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
              <svg class="w-10 h-10 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.interactive.selectProject') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center mb-6">{{ t('onboarding.interactive.selectProjectDescMulti') }}</p>

          <div class="space-y-3 mb-4">
            <button @click="doScanProjects" :disabled="isScanning" class="w-full px-4 py-2.5 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium disabled:opacity-50">
              {{ isScanning ? t('common.loading') : t('onboarding.interactive.scanProjects') }}
            </button>
            <button @click="doSelectProjectDir" :disabled="isScanning" class="w-full px-4 py-2.5 border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary rounded-lg hover:bg-gray-50 dark:hover:bg-surface-hover text-sm disabled:opacity-50">
              {{ t('onboarding.interactive.selectProjectDir') }}
            </button>
          </div>

          <div v-if="scannedProjects.length > 0" class="max-h-48 overflow-y-auto space-y-1.5 mb-4">
            <div
              v-for="project in scannedProjects"
              :key="project.project_id"
              @click="toggleProjectSelection(project.project_id)"
              :class="[
                'p-3 rounded-lg cursor-pointer transition-colors border',
                selectedProjectIds.has(project.project_id)
                  ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                  : 'border-gray-200 dark:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-hover/50'
              ]"
            >
              <div class="flex items-center gap-2">
                <div :class="['w-4 h-4 rounded border flex items-center justify-center flex-shrink-0', selectedProjectIds.has(project.project_id) ? 'bg-primary-600 border-primary-600' : 'border-gray-300 dark:border-surface-border']">
                  <svg v-if="selectedProjectIds.has(project.project_id)" class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <div class="min-w-0">
                  <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ project.name }}</p>
                  <p class="text-xs text-gray-500 dark:text-content-muted truncate">{{ project.path }}</p>
                </div>
              </div>
            </div>
          </div>

          <p v-if="selectedProjectIds.size > 0" class="text-xs text-primary-600 dark:text-primary-400 text-center">
            {{ t('onboarding.interactive.selectedCount', { count: selectedProjectIds.size }) }}
          </p>
          <p v-if="errorMessage" class="text-xs text-red-500 dark:text-red-400 text-center">{{ errorMessage }}</p>
        </template>

        <!-- Step 2: Import Plugins (multi-select) -->
        <template v-if="currentStep === 2">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center">
              <svg class="w-10 h-10 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.interactive.importPlugin') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center mb-6">{{ t('onboarding.interactive.importPluginDesc') }}</p>

          <div v-if="isScanning" class="text-center py-4">
            <div class="animate-spin w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full mx-auto mb-2"></div>
            <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('common.loading') }}</p>
          </div>

          <div v-else-if="scannedPlugins.length > 0" class="max-h-48 overflow-y-auto space-y-1.5 mb-4">
            <div
              v-for="plugin in scannedPlugins"
              :key="plugin.path"
              @click="togglePluginSelection(plugin.path)"
              :class="[
                'p-3 rounded-lg cursor-pointer transition-colors border',
                selectedPluginPaths.has(plugin.path)
                  ? 'border-green-500 bg-green-50 dark:bg-green-900/20'
                  : 'border-gray-200 dark:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-hover/50'
              ]"
            >
              <div class="flex items-center gap-2">
                <div :class="['w-4 h-4 rounded border flex items-center justify-center', selectedPluginPaths.has(plugin.path) ? 'bg-green-500 border-green-500' : 'border-gray-300 dark:border-surface-border']">
                  <svg v-if="selectedPluginPaths.has(plugin.path)" class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <div>
                  <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ plugin.plugin_name }}</p>
                  <p class="text-xs text-gray-500 dark:text-content-muted">{{ plugin.project_name }}</p>
                </div>
              </div>
            </div>
          </div>

          <div v-else class="text-center py-4">
            <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('onboarding.interactive.noPluginsFound') }}</p>
          </div>

          <p v-if="errorMessage" class="text-xs text-red-500 dark:text-red-400 text-center">{{ errorMessage }}</p>
        </template>

        <!-- Step 3: Configure Bindings (matrix) -->
        <template v-if="currentStep === 3">
          <div class="flex justify-center mb-4">
            <div class="w-16 h-16 rounded-full bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center">
              <svg class="w-8 h-8 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
              </svg>
            </div>
          </div>
          <h2 class="text-lg font-bold text-gray-900 dark:text-content-primary text-center mb-2">{{ t('onboarding.interactive.configureBindings') }}</h2>
          <p class="text-xs text-gray-600 dark:text-content-muted text-center mb-4">{{ t('onboarding.interactive.configureBindingsDesc') }}</p>

          <div class="border border-gray-200 dark:border-surface-border rounded-lg overflow-hidden">
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="bg-gray-50 dark:bg-surface-base/50">
                    <th class="text-left px-3 py-2 text-xs font-medium text-gray-500 dark:text-content-muted sticky left-0 bg-gray-50 dark:bg-surface-base/50 min-w-[100px]">
                      {{ t('onboarding.interactive.projectLabel') }}
                    </th>
                    <th
                      v-for="plugin in importedPlugins"
                      :key="plugin.plugin_id"
                      class="px-2 py-2 text-xs font-medium text-gray-500 dark:text-content-muted text-center min-w-[80px] max-w-[120px]"
                    >
                      <span class="block truncate" :title="plugin.name">{{ plugin.name }}</span>
                    </th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="project in scannedProjects.filter(p => selectedProjectIds.has(p.project_id))"
                    :key="project.project_id"
                    class="border-t border-gray-100 dark:border-surface-border"
                  >
                    <td class="px-3 py-2 sticky left-0 bg-white dark:bg-surface-card">
                      <div class="flex items-center gap-1.5">
                        <span class="text-sm font-medium text-gray-900 dark:text-content-primary truncate max-w-[80px]" :title="project.name">{{ project.name }}</span>
                        <span class="text-xs text-gray-400">({{ getBindingCount(project.project_id) }})</span>
                      </div>
                      <div class="flex gap-1 mt-0.5">
                        <button @click="selectAllForProject(project.project_id)" class="text-xs text-primary-600 dark:text-primary-400 hover:underline">{{ t('onboarding.interactive.selectAll') }}</button>
                        <button @click="deselectAllForProject(project.project_id)" class="text-xs text-gray-400 hover:underline">{{ t('onboarding.interactive.deselectAll') }}</button>
                      </div>
                    </td>
                    <td
                      v-for="plugin in importedPlugins"
                      :key="plugin.plugin_id"
                      class="px-2 py-2 text-center"
                    >
                      <input
                        type="checkbox"
                        :checked="isPluginBoundToProject(project.project_id, plugin.plugin_id)"
                        @change="toggleBinding(project.project_id, plugin.plugin_id)"
                        class="w-4 h-4 text-primary-600 rounded cursor-pointer"
                      />
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <p class="text-xs text-gray-500 dark:text-content-muted text-center mt-3">
            {{ t('onboarding.interactive.totalBindings', { count: totalBindings }) }}
          </p>
          <p v-if="errorMessage" class="text-xs text-red-500 dark:text-red-400 text-center">{{ errorMessage }}</p>
        </template>

        <!-- Step 4: Apply -->
        <template v-if="currentStep === 4">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
              <svg class="w-10 h-10 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.interactive.bindAndApply') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center mb-6">{{ t('onboarding.interactive.bindAndApplyDesc') }}</p>

          <div v-if="!applyResult" class="text-center">
            <button @click="doBindAndApply" :disabled="isApplying" class="px-6 py-2.5 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium disabled:opacity-50">
              <span v-if="isApplying">{{ t('onboarding.interactive.applying') }}</span>
              <span v-else>{{ t('onboarding.interactive.bindAndApplyButton') }}</span>
            </button>
          </div>

          <div v-else class="text-center">
            <div class="w-16 h-16 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center mx-auto mb-4">
              <svg class="w-8 h-8 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <p class="text-sm font-medium text-green-600 dark:text-green-400 mb-2">{{ t('onboarding.interactive.success') }}</p>
            <p class="text-xs text-gray-500 dark:text-content-muted">
              {{ t('onboarding.interactive.successDesc', { count: totalBindings }) }}
            </p>
          </div>

          <p v-if="errorMessage" class="text-xs text-red-500 dark:text-red-400 text-center mt-2">{{ errorMessage }}</p>
        </template>

        <!-- Navigation -->
        <div class="flex items-center justify-between mt-6">
          <button
            @click="skip"
            class="text-sm text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-gray-200"
          >
            {{ t('onboarding.skip') }}
          </button>
          <div class="flex items-center gap-3">
            <div class="flex gap-1.5">
              <div
                v-for="(_, idx) in steps"
                :key="idx"
                :class="[
                  'w-2 h-2 rounded-full transition-colors',
                  idx === currentStep ? 'bg-primary-600' : idx < currentStep ? 'bg-primary-300' : 'bg-gray-300 dark:bg-surface-layer'
                ]"
              />
            </div>
            <button
              v-if="currentStep === 2 && selectedPluginPaths.size > 0"
              @click="doImportPlugins"
              :disabled="isImporting"
              class="btn-primary text-sm font-medium disabled:opacity-50"
            >
              {{ isImporting ? t('common.loading') : t('onboarding.interactive.importSelected') }}
            </button>
            <template v-else-if="currentStep === 3">
              <button
                v-if="totalBindings > 0"
                @click="currentStep = 4"
                class="btn-primary text-sm font-medium"
              >
                {{ t('onboarding.next') }}
              </button>
              <button
                @click="finish"
                class="px-4 py-2 text-sm font-medium"
                :class="totalBindings > 0 ? 'text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-gray-200' : 'bg-primary-600 text-white rounded-lg hover:bg-primary-700'"
              >
                {{ totalBindings > 0 ? t('onboarding.interactive.skipBinding') : t('onboarding.startUsing') }}
              </button>
            </template>
            <button
              v-else-if="isLastStep && applyResult"
              @click="finish"
              class="btn-primary text-sm font-medium"
            >
              {{ t('onboarding.startUsing') }}
            </button>
            <button
              v-else-if="currentStep === 0 || currentStep === 1"
              @click="next"
              class="btn-primary text-sm font-medium"
            >
              {{ currentStep === 1 && selectedProjectIds.size === 0 ? t('onboarding.startUsing') : t('onboarding.next') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
