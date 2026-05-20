<script setup lang="ts">
import { computed } from 'vue'
import { useAutoSetup, type AutoSetupStep } from '@/composables/useAutoSetup'
import { useI18n } from 'vue-i18n'

const { isRunning, currentStep, stepMessage, progressPercent, lastResult, stepIndex } = useAutoSetup()
const { t } = useI18n()

const SETUP_STEPS: AutoSetupStep[] = ['scanning-projects', 'scanning-plugins', 'importing-plugins', 'binding-plugins', 'applying-changes']

const stepItems = computed(() =>
  SETUP_STEPS.map((step, i) => ({
    key: step,
    label: t(`autoSetup.stepNames.${step}`),
    completed: stepIndex.value > i,
    active: stepIndex.value === i,
  }))
)
</script>

<template>
  <Teleport to="body">
    <Transition name="auto-setup-slide">
      <div
        v-if="isRunning || currentStep === 'done'"
        class="fixed bottom-8 left-1/2 -translate-x-1/2 z-[80] w-full max-w-xl"
      >
        <div class="bg-white dark:bg-surface-card rounded-xl shadow-2xl border border-gray-200 dark:border-surface-border overflow-hidden">
          <div class="px-4 py-3">
            <div class="flex items-center gap-3">
              <div v-if="isRunning" class="animate-spin rounded-full h-5 w-5 border-2 border-primary-600 border-t-transparent shrink-0"></div>
              <svg v-else class="h-5 w-5 text-green-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">
                  {{ isRunning ? stepMessage : t('autoSetup.complete', { projects: lastResult?.projectsScanned ?? 0, plugins: lastResult?.pluginsImported ?? 0, bindings: lastResult?.bindingsCreated ?? 0 }) }}
                </p>
                <div v-if="isRunning" class="mt-1.5 w-full bg-gray-200 dark:bg-surface-hover rounded-full h-1.5">
                  <div
                    class="bg-primary-600 h-1.5 rounded-full transition-all duration-500 ease-out"
                    :style="{ width: `${progressPercent}%` }"
                  ></div>
                </div>
              </div>
              <span v-if="isRunning" class="text-xs text-gray-500 dark:text-content-muted shrink-0">{{ progressPercent }}%</span>
            </div>
          </div>

          <div v-if="isRunning" class="px-4 pb-3 pt-0">
            <div class="flex items-center gap-1">
              <template v-for="(item, i) in stepItems" :key="item.key">
                <div class="flex items-center gap-1 min-w-0">
                  <div
                    :class="[
                      'w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-medium shrink-0 transition-colors',
                      item.completed
                        ? 'bg-green-500 text-white'
                        : item.active
                          ? 'bg-primary-600 text-white ring-2 ring-primary-200 dark:ring-primary-800'
                          : 'bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400'
                    ]"
                  >
                    <svg v-if="item.completed" class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                    </svg>
                    <span v-else>{{ i + 1 }}</span>
                  </div>
                  <span
                    :class="[
                      'text-[11px] whitespace-nowrap transition-colors',
                      item.active
                        ? 'text-primary-700 dark:text-primary-400 font-medium'
                        : item.completed
                          ? 'text-green-600 dark:text-green-400'
                          : 'text-gray-400 dark:text-gray-500'
                    ]"
                  >{{ item.label }}</span>
                </div>
                <svg
                  v-if="i < stepItems.length - 1"
                  class="w-3 h-3 shrink-0 mx-0.5"
                  :class="item.completed ? 'text-green-400' : 'text-gray-300 dark:text-gray-600'"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </template>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.auto-setup-slide-enter-active {
  transition: all 0.3s ease-out;
}
.auto-setup-slide-leave-active {
  transition: all 0.3s ease-in;
}
.auto-setup-slide-enter-from {
  opacity: 0;
  transform: translate(-50%, 20px);
}
.auto-setup-slide-leave-to {
  opacity: 0;
  transform: translate(-50%, 20px);
}
</style>
