<script setup lang="ts">
import { useAutoSetup } from '@/composables/useAutoSetup'
import { useI18n } from 'vue-i18n'

const { isRunning, currentStep, stepMessage, progressPercent, lastResult } = useAutoSetup()
const { t } = useI18n()
</script>

<template>
  <Teleport to="body">
    <Transition name="auto-setup-slide">
      <div
        v-if="isRunning || currentStep === 'done'"
        class="fixed bottom-8 left-1/2 -translate-x-1/2 z-[80] w-full max-w-lg"
      >
        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl border border-gray-200 dark:border-gray-700 overflow-hidden">
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
                <div v-if="isRunning" class="mt-1.5 w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
                  <div
                    class="bg-primary-600 h-1.5 rounded-full transition-all duration-500 ease-out"
                    :style="{ width: `${progressPercent}%` }"
                  ></div>
                </div>
              </div>
              <span v-if="isRunning" class="text-xs text-gray-500 dark:text-gray-400 shrink-0">{{ progressPercent }}%</span>
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
