<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { api } from '@/api'
import { useOnboarding } from '@/composables/useOnboarding'
import { useLanguageDialog } from '@/composables/useLanguageDialog'
import { useAutoSetup } from '@/composables/useAutoSetup'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const currentStep = ref(0)
const { isVisible, hideOnboarding } = useOnboarding()
const { isVisible: languageDialogVisible } = useLanguageDialog()
const { runAutoSetup } = useAutoSetup()

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
  { title: t('onboarding.plugin.title'), icon: 'plugin' },
  { title: t('onboarding.project.title'), icon: 'project' },
  { title: t('onboarding.shortcuts.title'), icon: 'shortcuts' },
])

const progress = computed(() => ((currentStep.value + 1) / steps.value.length) * 100)

const markOnboardingCompleted = async () => {
  try {
    const settings = await api.getSettings()
    settings.onboarding_completed = true
    await api.saveSettings(settings)
  } catch {}
}

const prev = () => {
  if (currentStep.value > 0) {
    currentStep.value--
  }
}

const next = () => {
  if (currentStep.value < steps.value.length - 1) {
    currentStep.value++
  } else {
    finish()
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
    <div class="dialog-container w-full max-w-lg overflow-hidden">
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
            <div class="w-20 h-20 rounded-full bg-primary-100 dark:bg-surface-hover flex items-center justify-center">
              <svg class="w-10 h-10 text-primary-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.welcome.title') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center whitespace-pre-line">{{ t('onboarding.welcome.desc') }}</p>
        </template>

        <!-- Step 1: Plugin Management -->
        <template v-if="currentStep === 1">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center">
              <svg class="w-10 h-10 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.plugin.title') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center whitespace-pre-line">{{ t('onboarding.plugin.desc') }}</p>
        </template>

        <!-- Step 2: Projects & Engines -->
        <template v-if="currentStep === 2">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-blue-100 dark:bg-surface-hover flex items-center justify-center">
              <svg class="w-10 h-10 text-blue-600 dark:text-brand-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.project.title') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center whitespace-pre-line">{{ t('onboarding.project.desc') }}</p>
        </template>

        <!-- Step 3: Shortcuts -->
        <template v-if="currentStep === 3">
          <div class="flex justify-center mb-6">
            <div class="w-20 h-20 rounded-full bg-purple-100 dark:bg-surface-hover flex items-center justify-center">
              <svg class="w-10 h-10 text-purple-600 dark:text-content-secondary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
            </div>
          </div>
          <h2 class="text-xl font-bold text-gray-900 dark:text-content-primary text-center mb-3">{{ t('onboarding.shortcuts.title') }}</h2>
          <p class="text-sm text-gray-600 dark:text-content-muted text-center mb-6">{{ t('onboarding.shortcuts.desc') }}</p>
          <div class="space-y-3">
            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-surface-base/50 rounded">
              <div class="flex items-center gap-3">
                <kbd class="px-2 py-1 text-xs font-mono bg-white dark:bg-surface-card border border-gray-200 dark:border-surface-border rounded shadow-sm">{{ t('onboarding.shortcuts.ctrlK') }}</kbd>
                <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('onboarding.shortcuts.ctrlKDesc') }}</span>
              </div>
            </div>
            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-surface-base/50 rounded">
              <div class="flex items-center gap-3">
                <kbd class="px-2 py-1 text-xs font-mono bg-white dark:bg-surface-card border border-gray-200 dark:border-surface-border rounded shadow-sm">{{ t('onboarding.shortcuts.ctrlB') }}</kbd>
                <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('onboarding.shortcuts.ctrlBDesc') }}</span>
              </div>
            </div>
            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-surface-base/50 rounded">
              <div class="flex items-center gap-3">
                <kbd class="px-2 py-1 text-xs font-mono bg-white dark:bg-surface-card border border-gray-200 dark:border-surface-border rounded shadow-sm">{{ t('onboarding.shortcuts.ctrlT') }}</kbd>
                <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('onboarding.shortcuts.ctrlTDesc') }}</span>
              </div>
            </div>
            <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-surface-base/50 rounded">
              <div class="flex items-center gap-3">
                <kbd class="px-2 py-1 text-xs font-mono bg-white dark:bg-surface-card border border-gray-200 dark:border-surface-border rounded shadow-sm">{{ t('onboarding.shortcuts.ctrlD') }}</kbd>
                <span class="text-sm text-gray-700 dark:text-content-secondary">{{ t('onboarding.shortcuts.ctrlDDesc') }}</span>
              </div>
            </div>
          </div>
        </template>

        <!-- Navigation -->
        <div class="flex items-center justify-between mt-8">
          <button
            v-if="currentStep > 0"
            @click="prev"
            class="text-sm text-gray-500 dark:text-content-muted hover:text-gray-700 dark:hover:text-gray-200"
          >
            {{ t('onboarding.prev') }}
          </button>
          <button
            v-else
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
            <button v-if="currentStep < steps.length - 1" @click="next" class="btn-primary text-sm font-medium">
              {{ t('onboarding.next') }}
            </button>
            <button v-else @click="next" class="btn-primary text-sm font-medium">
              {{ t('onboarding.startUsing') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
