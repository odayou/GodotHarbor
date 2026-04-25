<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { api } from '@/api'
import type { Engine } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useDialogEscape } from '@/composables/useDialogEscape'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const toast = useToast()
const engines = ref<Engine[]>([])
const isLoading = ref(false)
const showAddDialog = ref(false)
const newEnginePath = ref('')
const newEngineName = ref('')
const isRegistering = ref(false)
const showDeleteConfirm = ref(false)
const deleteTargetId = ref('')
let unlistenDiscover: UnlistenFn | null = null

useDialogEscape(showAddDialog)

onMounted(async () => {
  loadEngines()
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

const loadEngines = async () => {
  isLoading.value = true
  try {
    const result = await api.getEngines()
    engines.value = result
  } catch (error) {
    toast.error(`加载引擎列表失败: ${error}`)
  } finally {
    isLoading.value = false
  }
}

const selectEnginePath = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Godot 引擎目录'
    })
    if (selected && typeof selected === 'string') {
      newEnginePath.value = selected
    }
  } catch (error) {
    toast.error(`选择目录失败: ${error}`)
  }
}

const registerEngine = async () => {
  if (!newEnginePath.value) {
    toast.warning('请选择引擎目录')
    return
  }
  isRegistering.value = true
  try {
    const result = await api.registerEngine(newEnginePath.value, newEngineName.value)
    toast.success(`成功注册引擎: ${result.name}`)
    showAddDialog.value = false
    newEnginePath.value = ''
    newEngineName.value = ''
    await loadEngines()
  } catch (error) {
    toast.error(`注册引擎失败: ${error}`)
  } finally {
    isRegistering.value = false
  }
}

const confirmRemoveEngine = (engineId: string) => {
  deleteTargetId.value = engineId
  showDeleteConfirm.value = true
}

const onRemoveEngineConfirm = async () => {
  try {
    await api.removeEngine(deleteTargetId.value)
    toast.success('引擎已删除')
    await loadEngines()
  } catch (error) {
    toast.error(`删除引擎失败: ${error}`)
  }
}

const setDefault = async (engineId: string) => {
  try {
    await api.setDefaultEngine(engineId)
    toast.success('已将选中的引擎设为默认')
    await loadEngines()
  } catch (error) {
    toast.error(`设置默认引擎失败: ${error}`)
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">引擎管理</h1>
      <button
        @click="showAddDialog = true"
        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
      >
        注册新引擎
      </button>
    </div>

    <div class="bg-white dark:bg-gray-800 rounded-xl shadow p-5">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <svg class="w-5 h-5 text-primary-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          <span class="text-sm text-gray-600 dark:text-gray-400">默认引擎:</span>
        </div>
        <span v-if="defaultEngine" class="text-sm font-medium text-gray-900 dark:text-gray-100">
          {{ defaultEngine.name }} (v{{ defaultEngine.version }})
        </span>
        <span v-else class="text-sm text-yellow-600 dark:text-yellow-400">
          未设置默认引擎
        </span>
      </div>
    </div>

    <div v-if="isLoading" class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>

    <div v-else-if="engines.length === 0" class="text-center py-12">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-gray-100">暂无已注册的引擎</h3>
      <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        注册 Godot 引擎以启动项目
      </p>
      <div class="mt-4 flex justify-center">
        <button
          @click="showAddDialog = true"
          class="inline-flex items-center gap-1.5 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors text-sm"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          注册引擎
        </button>
      </div>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div
        v-for="engine in engines"
        :key="engine.engine_id"
        :class="[
          'bg-white dark:bg-gray-800 rounded-xl shadow p-5 border-2 transition-colors',
          engine.is_default ? 'border-primary-500' : 'border-transparent hover:border-gray-200 dark:hover:border-gray-600'
        ]"
      >
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center">
              <svg class="w-6 h-6 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
                  {{ engine.name }}
                </h3>
                <span
                  v-if="engine.is_default"
                  class="px-2 py-0.5 rounded text-xs font-medium bg-primary-100 text-primary-800 dark:bg-primary-900/30 dark:text-primary-400"
                >
                  默认
                </span>
              </div>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
                v{{ engine.version }}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-1">
            <button
              v-if="!engine.is_default"
              @click="setDefault(engine.engine_id)"
              class="text-primary-600 hover:text-primary-800 dark:text-primary-400 p-1"
              title="设为默认"
            >
              <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
              </svg>
            </button>
            <button
              @click="confirmRemoveEngine(engine.engine_id)"
              class="text-red-600 hover:text-red-800 p-1"
            >
              <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
        <div class="mt-4 space-y-2">
          <div class="flex items-center gap-2 text-sm">
            <span class="text-gray-500 dark:text-gray-400">类型:</span>
            <span class="px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300">
              {{ engine.engine_type === 'Godot4' ? 'Godot 4' : engine.engine_type === 'Godot3' ? 'Godot 3' : '未知' }}
            </span>
          </div>
          <div class="text-sm text-gray-500 dark:text-gray-400 truncate" :title="engine.path">
            {{ engine.path }}
          </div>
        </div>
      </div>
    </div>

    <div v-if="showAddDialog" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showAddDialog = false; newEnginePath = ''; newEngineName = ''">
      <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl" @click.stop>
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">注册新引擎</h3>
        <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
          选择 Godot 引擎的安装目录（包含 godot.exe 或 godot 可执行文件的目录）
        </p>
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">引擎名称（可选）</label>
            <input
              v-model="newEngineName"
              type="text"
              placeholder="留空则使用目录名"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">引擎路径</label>
            <div class="flex gap-2">
              <input
                v-model="newEnginePath"
                type="text"
                readonly
                placeholder="请选择引擎目录"
                class="flex-1 px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-gray-50 dark:bg-gray-700 text-gray-900 dark:text-gray-100 text-sm"
              />
              <button
                @click="selectEnginePath"
                class="px-4 py-2 bg-gray-100 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-200 dark:hover:bg-gray-500 text-sm whitespace-nowrap"
              >
                浏览
              </button>
            </div>
          </div>
        </div>
        <div class="flex justify-end space-x-3 mt-6">
          <button
            @click="showAddDialog = false; newEnginePath = ''; newEngineName = ''"
            class="px-4 py-2 bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-500"
          >
            取消
          </button>
          <button
            @click="registerEngine"
            :disabled="isRegistering || !newEnginePath"
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {{ isRegistering ? '注册中...' : '注册' }}
          </button>
        </div>
      </div>
    </div>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      title="确认删除引擎"
      :description="`确定要删除引擎吗？此操作不可撤销。`"
      confirm-text="确认删除"
      @confirm="onRemoveEngineConfirm"
    />
  </div>
</template>
