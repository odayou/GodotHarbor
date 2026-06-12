<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { OneClickInstallResult } from '@/types'

const props = defineProps<{
  visible: boolean
  pluginName: string
  result: OneClickInstallResult | null
  isInstalling: boolean
  currentStage: 'downloading' | 'importing' | 'binding' | 'applying' | 'complete' | 'error' | null
  stageMessage: string
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const { t } = useI18n()

const steps = computed(() => [
  {
    key: 'downloading',
    label: t('assetLibrary.stepDownloading'),
    done: props.result !== null || (props.currentStage !== null && ['importing', 'binding', 'applying', 'complete'].includes(props.currentStage)),
    active: props.currentStage === 'downloading',
    error: props.currentStage === 'error' && props.result === null,
  },
  {
    key: 'importing',
    label: t('assetLibrary.stepImporting'),
    done: props.result !== null || (props.currentStage !== null && ['binding', 'applying', 'complete'].includes(props.currentStage)),
    active: props.currentStage === 'importing',
    error: false,
  },
  {
    key: 'binding',
    label: t('assetLibrary.stepBinding'),
    done: props.result !== null && props.result.binding_created,
    active: props.currentStage === 'binding',
    error: props.result !== null && !props.result.binding_created && props.result.errors.some(e => e.includes('绑定')),
  },
  {
    key: 'applying',
    label: t('assetLibrary.stepApplying'),
    done: props.result !== null && props.result.changes_applied,
    active: props.currentStage === 'applying',
    error: props.result !== null && !props.result.changes_applied && props.result.errors.some(e => e.includes('应用')),
  },
])
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click.self="!isInstalling && emit('close')">
      <div class="dialog-container w-full max-w-md" @click.stop>
        <div class="flex justify-between items-center mb-5">
          <h3 class="dialog-title mb-0">{{ t('assetLibrary.oneClickInstallTitle') }}</h3>
          <button
            v-if="!isInstalling"
            @click="emit('close')"
            class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
          >
            <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div class="mb-4">
          <p class="text-sm text-gray-700 dark:text-content-secondary">{{ pluginName }}</p>
        </div>

        <div class="space-y-3">
          <div
            v-for="step in steps"
            :key="step.key"
            :class="[
              'flex items-center gap-3 p-3 rounded transition-colors',
              step.active ? 'bg-primary-50 dark:bg-primary-900/20' :
              step.done ? 'bg-green-50 dark:bg-green-900/20' :
              step.error ? 'bg-red-50 dark:bg-red-900/20' :
              'bg-gray-50 dark:bg-surface-layer'
            ]"
          >
            <div class="w-8 h-8 rounded-full flex items-center justify-center text-sm flex-shrink-0"
              :class="[
                step.done ? 'bg-green-100 dark:bg-green-900/40' :
                step.active ? 'bg-primary-100 dark:bg-primary-900/40 animate-pulse' :
                step.error ? 'bg-red-100 dark:bg-red-900/40' :
                'bg-gray-200 dark:bg-gray-700'
              ]"
            >
              <svg v-if="step.done" class="w-4 h-4 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
              </svg>
              <svg v-else-if="step.error" class="w-4 h-4 text-red-600 dark:text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
              </svg>
              <svg v-else-if="step.active" class="w-4 h-4 text-primary-600 dark:text-primary-400 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              <span v-else class="text-gray-400 dark:text-content-muted text-xs">{{ steps.indexOf(step) + 1 }}</span>
            </div>

            <div class="flex-1">
              <p :class="[
                'text-sm font-medium',
                step.done ? 'text-green-700 dark:text-green-400' :
                step.active ? 'text-primary-700 dark:text-primary-400' :
                step.error ? 'text-red-700 dark:text-red-400' :
                'text-gray-500 dark:text-content-muted'
              ]">{{ step.label }}</p>
            </div>
          </div>
        </div>

        <div v-if="stageMessage && isInstalling" class="mt-3 text-xs text-gray-500 dark:text-content-muted text-center">
          {{ stageMessage }}
        </div>

        <div v-if="result && !isInstalling" class="mt-4">
          <div v-if="result.success" class="p-3 bg-green-50 dark:bg-green-900/20 rounded">
            <p class="text-sm text-green-700 dark:text-green-400 font-medium">{{ t('assetLibrary.installSuccess') }}</p>
            <div v-if="result.binding_created" class="text-xs text-green-600 dark:text-green-400 mt-1">{{ t('assetLibrary.bindingCreated') }}</div>
            <div v-if="result.changes_applied" class="text-xs text-green-600 dark:text-green-400">{{ t('assetLibrary.changesApplied') }}</div>
          </div>
          <div v-else class="p-3 bg-red-50 dark:bg-red-900/20 rounded">
            <p class="text-sm text-red-700 dark:text-red-400 font-medium">{{ t('assetLibrary.installFailed') }}</p>
            <div v-if="result.errors.length > 0" class="mt-1 space-y-0.5">
              <p v-for="(err, i) in result.errors" :key="i" class="text-xs text-red-600 dark:text-red-400">{{ err }}</p>
            </div>
          </div>
        </div>

        <div v-if="!isInstalling" class="flex justify-end mt-4 pt-3 border-t border-gray-200 dark:border-surface-border">
          <button @click="emit('close')" class="btn-secondary text-sm">
            {{ t('common.close') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
