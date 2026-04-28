<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import type { Engine } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const { t } = useI18n()
const engines = ref<Engine[]>([])
const isLoading = ref(false)
const showAddDialog = ref(false)
const newEnginePath = ref('')
const newEngineName = ref('')
const isRegistering = ref(false)
const showDeleteConfirm = ref(false)
const deleteTargetId = ref('')
const deleteBoundProjects = ref<string[]>([])
let unlistenDiscover: UnlistenFn | null = null

const searchQuery = ref('')
const filterType = ref<string>('all')
const engineHealthMap = ref<Map<string, boolean>>(new Map())
const boundProjectsMap = ref<Map<string, string[]>>(new Map())

const showRenameDialog = ref(false)
const renameEngineId = ref('')
const renameInput = ref('')

useDialogEscape(showAddDialog)
useDialogEscape(showRenameDialog)

onMounted(async () => {
  await loadEngines()
  unlistenDiscover = await listen('engines-discovered', () => {
    loadEngines()
  })
})

onUnmounted(() => {
  if (unlistenDiscover) {
    unlistenDiscover()
  }
})

const defaultEngine = computed(() => {
  return engines.value.find(e => e.is_default)
})

const filteredEngines = computed(() => {
  return engines.value.filter(engine => {
    const matchesSearch = searchQuery.value === '' ||
      engine.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      engine.version.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      engine.path.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesType = filterType.value === 'all' ||
      engine.engine_type === filterType.value

    return matchesSearch && matchesType
  })
})

const loadEngines = async () => {
  isLoading.value = true
  try {
    const result = await api.getEngines()
    engines.value = result
    await checkAllEngineHealth()
    await loadAllBoundProjects()
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const checkAllEngineHealth = async () => {
  const healthMap = new Map<string, boolean>()
  for (const engine of engines.value) {
    try {
      const healthy = await api.checkEngineHealth(engine.engine_id)
      healthMap.set(engine.engine_id, healthy)
    } catch {
      healthMap.set(engine.engine_id, false)
    }
  }
  engineHealthMap.value = healthMap
}

const loadAllBoundProjects = async () => {
  const projectsMap = new Map<string, string[]>()
  for (const engine of engines.value) {
    try {
      const projects = await api.getEngineBoundProjects(engine.engine_id)
      projectsMap.set(engine.engine_id, projects)
    } catch {
      projectsMap.set(engine.engine_id, [])
    }
  }
  boundProjectsMap.value = projectsMap
}

const discoverEngines = async () => {
  isLoading.value = true
  try {
    const discovered = await api.autoDiscoverEngines()
    if (discovered.length > 0) {
      toast.success(t('engines.discoveredCount', { count: discovered.length }))
      await loadEngines()
    } else {
      toast.info(t('engines.noNewEngines'))
    }
  } catch (error) {
    toast.error(t('engines.discoverFailed', { error }))
  } finally {
    isLoading.value = false
  }
}

const selectEnginePath = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('engines.selectEngineDir')
    })
    if (selected && typeof selected === 'string') {
      newEnginePath.value = selected
    }
  } catch (error) {
    toast.error(t('common.selectDirFailed', { error }))
  }
}

const registerEngine = async () => {
  if (!newEnginePath.value) {
    toast.warning(t('engines.selectEngineDirFirst'))
    return
  }
  isRegistering.value = true
  try {
    const result = await api.registerEngine(newEnginePath.value, newEngineName.value)
    toast.success(t('engines.registerSuccess', { name: result.name }))
    showAddDialog.value = false
    newEnginePath.value = ''
    newEngineName.value = ''
    await loadEngines()
  } catch (error) {
    toast.error(t('engines.registerFailed', { error }))
  } finally {
    isRegistering.value = false
  }
}

const confirmRemoveEngine = async (engineId: string) => {
  deleteTargetId.value = engineId
  try {
    deleteBoundProjects.value = await api.getEngineBoundProjects(engineId)
  } catch {
    deleteBoundProjects.value = []
  }
  showDeleteConfirm.value = true
}

const onRemoveEngineConfirm = async () => {
  try {
    await api.removeEngine(deleteTargetId.value)
    toast.success(t('engines.deleteSuccess'))
    await loadEngines()
  } catch (error) {
    toast.error(t('common.deleteFailed', { error }))
  }
}

const setDefault = async (engineId: string) => {
  try {
    await api.setDefaultEngine(engineId)
    toast.success(t('engines.defaultSet'))
    await loadEngines()
  } catch (error) {
    toast.error(t('common.loadFailed', { error }))
  }
}

const openInFileManager = async (path: string) => {
  try {
    await api.openInFileManager(path)
  } catch (error) {
    toast.error(t('engines.openInFileManagerFailed', { error }))
  }
}

const openRenameDialog = (engine: Engine) => {
  renameEngineId.value = engine.engine_id
  renameInput.value = engine.name
  showRenameDialog.value = true
}

const saveRename = async () => {
  if (!renameInput.value.trim()) {
    toast.warning(t('engines.nameRequired'))
    return
  }
  try {
    await api.renameEngine(renameEngineId.value, renameInput.value)
    toast.success(t('engines.renameSuccess'))
    showRenameDialog.value = false
    await loadEngines()
  } catch (error) {
    toast.error(t('engines.renameFailed', { error }))
  }
}

const checkEngineUpdates = async () => {
  try {
    const result = await api.checkGodotUpdates()
    if (result.updates_available.length > 0) {
      toast.info(t('engines.updatesAvailable', { count: result.updates_available.length }))
    } else {
      toast.success(t('engines.noUpdates'))
    }
  } catch (error) {
    toast.error(t('engines.checkUpdatesFailed', { error }))
  }
}
</script>

<template>
  <div class="relative">
    <div class="space-y-6">
      <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">{{ t('engines.title') }}</h1>
      <div class="flex flex-wrap gap-2">
        <button
          @click="discoverEngines"
          :disabled="isLoading"
          class="px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors disabled:opacity-50 text-sm"
        >
          {{ t('engines.discover') }}
        </button>
        <button
          @click="showAddDialog = true"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
        >
          {{ t('engines.register') }}
        </button>
      </div>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <svg class="w-5 h-5 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('engines.defaultEngine') }}:</span>
          </div>
          <span v-if="defaultEngine" class="text-sm font-medium text-gray-900 dark:text-gray-100">
            {{ defaultEngine.name }} (v{{ defaultEngine.version }})
          </span>
          <span v-else class="text-sm text-yellow-600 dark:text-yellow-400">
            {{ t('engines.noDefaultEngine') }}
          </span>
        </div>
        <button
          @click="checkEngineUpdates"
          class="px-3 py-1.5 text-xs font-medium border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors"
        >
          {{ t('engines.checkUpdates') }}
        </button>
      </div>
    </div>

    <div class="card">
      <div class="flex flex-col lg:flex-row gap-4">
        <div class="flex-1">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('engines.search')"
            class="w-full px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          />
        </div>
        <div class="flex gap-2 items-center">
          <select
            v-model="filterType"
            class="px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
          >
            <option value="all">{{ t('engines.allTypes') }}</option>
            <option value="Godot4">Godot 4</option>
            <option value="Godot3">Godot 3</option>
            <option value="Unknown">{{ t('engines.unknown') }}</option>
          </select>
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="engines.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('engines.empty') }}</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        {{ t('engines.emptyDesc') }}
      </p>
      <div class="mt-4 flex justify-center gap-3">
        <button
          @click="discoverEngines"
          :disabled="isLoading"
          class="inline-flex items-center gap-1.5 px-4 py-2 border border-primary-600 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          {{ t('engines.discover') }}
        </button>
        <button
          @click="showAddDialog = true"
          class="inline-flex items-center gap-1.5 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          {{ t('engines.register') }}
        </button>
      </div>
    </div>

    <div v-else-if="filteredEngines.length === 0" class="text-center py-12">
      <p class="text-sm text-gray-500 dark:text-gray-400">{{ t('engines.noMatchingEngines') }}</p>
    </div>

    <div v-else class="bg-white dark:bg-gray-800 rounded-xl shadow overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full min-w-[800px]">
          <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
            <tr
              v-for="engine in filteredEngines"
              :key="engine.engine_id"
              :class="[
                'hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors',
                engine.is_default ? 'bg-primary-50/50 dark:bg-primary-900/10' : ''
              ]"
            >
              <td class="px-4 py-4 whitespace-nowrap">
                <div class="flex items-center gap-3">
                  <div class="w-8 h-8 rounded-lg bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center">
                    <svg class="w-5 h-5 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
                    </svg>
                  </div>
                  <div>
                    <div class="flex items-center gap-2">
                      <span class="font-medium text-gray-900 dark:text-gray-100 text-sm">
                        {{ engine.name }}
                      </span>
                      <span
                        v-if="engine.is_default"
                        class="px-2 py-0.5 rounded text-xs font-medium bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-400"
                      >
                        {{ t('engines.default') }}
                      </span>
                      <span
                        v-if="engineHealthMap.get(engine.engine_id) === false"
                        class="px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
                        :title="t('engines.exeNotFound')"
                      >
                        ⚠️
                      </span>
                    </div>
                    <span class="text-xs text-gray-500 dark:text-gray-400">v{{ engine.version }}</span>
                  </div>
                </div>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <span class="px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300">
                  {{ engine.engine_type === 'Godot4' ? 'Godot 4' : engine.engine_type === 'Godot3' ? 'Godot 3' : t('engines.unknown') }}
                </span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <span
                  v-if="engineHealthMap.get(engine.engine_id) === true"
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400"
                >
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  {{ t('engines.healthy') }}
                </span>
                <span
                  v-else-if="engineHealthMap.get(engine.engine_id) === false"
                  class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400"
                >
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                  {{ t('engines.unhealthy') }}
                </span>
                <span v-else class="text-xs text-gray-400">{{ t('engines.checking') }}</span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <span class="text-sm text-gray-600 dark:text-gray-300">
                  {{ boundProjectsMap.get(engine.engine_id)?.length || 0 }}
                </span>
              </td>
              <td class="px-4 py-4">
                <span class="text-sm text-gray-500 dark:text-gray-400 truncate max-w-xs block" :title="engine.path">
                  {{ engine.path }}
                </span>
              </td>
              <td class="px-4 py-4 whitespace-nowrap">
                <div class="flex items-center justify-end gap-1">
                  <button
                    @click="openRenameDialog(engine)"
                    class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                    :title="t('engines.rename')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  <button
                    @click="openInFileManager(engine.path)"
                    class="text-gray-500 hover:text-primary-600 dark:hover:text-primary-400 p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-600 transition-colors"
                    :title="t('engines.openInFileManager')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                    </svg>
                  </button>
                  <button
                    v-if="!engine.is_default"
                    @click="setDefault(engine.engine_id)"
                    class="text-primary-600 hover:text-primary-800 dark:text-primary-400 p-2 rounded-lg hover:bg-primary-50 dark:hover:bg-primary-900/20 transition-colors"
                    :title="t('engines.setDefault')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                    </svg>
                  </button>
                  <button
                    @click="confirmRemoveEngine(engine.engine_id)"
                    class="text-red-500 hover:text-red-700 p-2 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                    :title="t('engines.deleteEngine')"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>

  <div v-if="showAddDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showAddDialog = false; newEnginePath = ''; newEngineName = ''">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('engines.registerTitle') }}</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          {{ t('engines.registerDesc') }}
        </p>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('engines.engineName') }}</label>
            <input
              v-model="newEngineName"
              type="text"
              :placeholder="t('engines.engineNamePlaceholder')"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{{ t('engines.enginePath') }}</label>
            <div class="flex gap-2">
              <input
                v-model="newEnginePath"
                type="text"
                readonly
                :placeholder="t('engines.enginePathPlaceholder')"
                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
              />
              <button
                @click="selectEnginePath"
                class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
              >
                {{ t('projects.browse') }}
              </button>
            </div>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showAddDialog = false; newEnginePath = ''; newEngineName = ''"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="registerEngine"
            :disabled="isRegistering || !newEnginePath"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ isRegistering ? t('engines.registering') : t('engines.register') }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="showRenameDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showRenameDialog = false">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('engines.renameTitle') }}</h3>
        <input
          v-model="renameInput"
          type="text"
          :placeholder="t('engines.engineNamePlaceholder')"
          class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
          @keyup.enter="saveRename"
        />
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showRenameDialog = false"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            {{ t('common.cancel') }}
          </button>
          <button
            @click="saveRename"
            :disabled="!renameInput.trim()"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ t('common.confirm') }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('engines.deleteConfirm')"
      :description="deleteBoundProjects.length > 0 
        ? t('engines.deleteConfirmDescWithProjects', { projects: deleteBoundProjects.join(', ') }) 
        : t('engines.deleteConfirmDesc')"
      :confirm-text="t('common.confirm')"
      @confirm="onRemoveEngineConfirm"
    />
  </div>
</template>
