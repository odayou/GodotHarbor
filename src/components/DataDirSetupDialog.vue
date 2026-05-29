<script setup lang="ts">
import { ref } from 'vue'
import { api } from '@/api'
import { useI18n } from 'vue-i18n'
import { useToast } from '@/composables/useToast'
import { open } from '@tauri-apps/plugin-dialog'

const { t } = useI18n()
const toast = useToast()

const visible = defineModel<boolean>('visible', { default: false })

const selectedPath = ref('')
const isConfirming = ref(false)

const selectDirectory = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('dataDirSetup.selectDir'),
    })
    if (selected) {
      selectedPath.value = typeof selected === 'string' ? selected : (selected as unknown as string)
    }
  } catch {}
}

const confirm = async () => {
  isConfirming.value = true
  try {
    const result = await api.confirmDataDir(selectedPath.value || undefined)
    toast.success(t('dataDirSetup.confirmed', { dir: result }))
    visible.value = false
  } catch (e) {
    toast.error(String(e))
  } finally {
    isConfirming.value = false
  }
}

const useDefault = async () => {
  isConfirming.value = true
  try {
    const result = await api.confirmDataDir()
    toast.success(t('dataDirSetup.confirmed', { dir: result }))
    visible.value = false
  } catch (e) {
    toast.error(String(e))
  } finally {
    isConfirming.value = false
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50">
      <div class="bg-white dark:bg-surface-card rounded-2xl shadow-2xl w-full max-w-lg mx-4 overflow-hidden">
        <div class="p-8">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
              <svg class="w-10 h-10 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
          </div>

          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-2">
            {{ t('dataDirSetup.title') }}
          </h2>
          <p class="text-sm text-gray-500 dark:text-content-muted text-center mb-6">
            {{ t('dataDirSetup.desc') }}
          </p>

          <div class="space-y-4">
            <div class="p-4 border border-gray-200 dark:border-surface-border rounded-xl hover:border-primary-300 dark:hover:border-primary-700 transition-colors cursor-pointer" @click="selectDirectory">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center shrink-0">
                  <svg class="w-5 h-5 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('dataDirSetup.customDir') }}</div>
                  <div v-if="selectedPath" class="text-xs text-primary-600 dark:text-primary-400 mt-0.5 truncate">{{ selectedPath }}</div>
                  <div v-else class="text-xs text-gray-400 dark:text-content-muted mt-0.5">{{ t('dataDirSetup.customDirHint') }}</div>
                </div>
              </div>
            </div>

            <div class="p-4 border border-gray-200 dark:border-surface-border rounded-xl hover:border-amber-300 dark:hover:border-amber-700 transition-colors cursor-pointer" @click="useDefault">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center shrink-0">
                  <svg class="w-5 h-5 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('dataDirSetup.useAppDir') }}</div>
                  <div class="text-xs text-gray-400 dark:text-content-muted mt-0.5">{{ t('dataDirSetup.useAppDirHint') }}</div>
                </div>
              </div>
            </div>
          </div>

          <div class="flex items-center justify-between mt-6">
            <p class="text-xs text-gray-400 dark:text-content-muted">{{ t('dataDirSetup.laterHint') }}</p>
            <button
              v-if="selectedPath"
              @click="confirm"
              :disabled="isConfirming"
              class="btn-primary text-sm font-medium disabled:opacity-50"
            >
              {{ isConfirming ? t('common.loading') : t('dataDirSetup.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>
