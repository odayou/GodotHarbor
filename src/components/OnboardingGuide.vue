<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import { useOnboarding } from '@/composables/useOnboarding'
import { useLanguageDialog } from '@/composables/useLanguageDialog'
import { useAutoSetup } from '@/composables/useAutoSetup'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const router = useRouter()
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
  {
    title: t('onboarding.welcome.title'),
    desc: t('onboarding.welcome.desc'),
    icon: 'welcome',
    action: null
  },
  {
    title: t('onboarding.scan.title'),
    desc: t('onboarding.scan.desc'),
    icon: 'scan',
    action: '/settings'
  },
  {
    title: t('onboarding.import.title'),
    desc: t('onboarding.import.desc'),
    icon: 'import',
    action: '/plugins'
  },
  {
    title: t('onboarding.shortcuts.title'),
    desc: t('onboarding.shortcuts.desc'),
    icon: 'shortcuts',
    action: null
  }
])

const currentStepData = computed(() => steps.value[currentStep.value])
const isLastStep = computed(() => currentStep.value === steps.value.length - 1)
const progress = computed(() => ((currentStep.value + 1) / steps.value.length) * 100)

const markOnboardingCompleted = async () => {
  try {
    const settings = await api.getSettings()
    settings.onboarding_completed = true
    await api.saveSettings(settings)
  } catch {}
}

const next = () => {
  if (isLastStep.value) {
    finish()
  } else {
    currentStep.value++
  }
}

const skip = () => {
  finish()
}

const goToStep = () => {
  if (currentStepData.value.action) {
    router.push(currentStepData.value.action)
  }
  finish()
}

const finish = async () => {
  hideOnboarding()
  currentStep.value = 0
  await markOnboardingCompleted()
  setTimeout(() => {
    runAutoSetup()
  }, 500)
}
</script>

<template>
  <div v-if="isVisible" class="fixed inset-0 bg-black/60 flex items-center justify-center z-[100] p-4">
    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-full max-w-lg overflow-hidden">
      <div class="bg-primary-600 h-1.5">
        <div
          class="bg-primary-400 h-full transition-all duration-300"
          :style="{ width: progress + '%' }"
        />
      </div>

      <div class="p-8">
        <div class="flex justify-center mb-6">
          <div v-if="currentStepData.icon === 'welcome'" class="w-20 h-20 rounded-full bg-primary-100 dark:bg-primary-900/30 flex items-center justify-center">
            <svg class="w-10 h-10 text-primary-600 dark:text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
            </svg>
          </div>
          <div v-else-if="currentStepData.icon === 'scan'" class="w-20 h-20 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
            <svg class="w-10 h-10 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </div>
          <div v-else-if="currentStepData.icon === 'import'" class="w-20 h-20 rounded-full bg-green-100 dark:bg-green-900/30 flex items-center justify-center">
            <svg class="w-10 h-10 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
          </div>
          <div v-else-if="currentStepData.icon === 'shortcuts'" class="w-20 h-20 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
            <svg class="w-10 h-10 text-amber-600 dark:text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
            </svg>
          </div>
        </div>

        <h2 class="text-xl font-bold text-gray-900 dark:text-gray-100 text-center mb-3">
          {{ currentStepData.title }}
        </h2>
        <p v-if="currentStepData.icon !== 'shortcuts'" class="text-sm text-gray-600 dark:text-gray-400 text-center whitespace-pre-line mb-8">
          {{ currentStepData.desc }}
        </p>
        <div v-else class="mb-8 space-y-2.5">
          <p class="text-sm text-gray-600 dark:text-gray-400 text-center mb-4">{{ currentStepData.desc }}</p>
          <div class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
            <kbd class="px-2 py-0.5 bg-white dark:bg-gray-600 border border-gray-200 dark:border-gray-500 rounded text-xs font-mono text-gray-800 dark:text-gray-200 shadow-sm">{{ t('onboarding.shortcuts.ctrlK') }}</kbd>
            <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('onboarding.shortcuts.ctrlKDesc') }}</span>
          </div>
          <div class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
            <kbd class="px-2 py-0.5 bg-white dark:bg-gray-600 border border-gray-200 dark:border-gray-500 rounded text-xs font-mono text-gray-800 dark:text-gray-200 shadow-sm">{{ t('onboarding.shortcuts.ctrlB') }}</kbd>
            <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('onboarding.shortcuts.ctrlBDesc') }}</span>
          </div>
          <div class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
            <kbd class="px-2 py-0.5 bg-white dark:bg-gray-600 border border-gray-200 dark:border-gray-500 rounded text-xs font-mono text-gray-800 dark:text-gray-200 shadow-sm">{{ t('onboarding.shortcuts.ctrlT') }}</kbd>
            <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('onboarding.shortcuts.ctrlTDesc') }}</span>
          </div>
          <div class="flex items-center justify-between px-4 py-2 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
            <kbd class="px-2 py-0.5 bg-white dark:bg-gray-600 border border-gray-200 dark:border-gray-500 rounded text-xs font-mono text-gray-800 dark:text-gray-200 shadow-sm">{{ t('onboarding.shortcuts.ctrlD') }}</kbd>
            <span class="text-sm text-gray-600 dark:text-gray-400">{{ t('onboarding.shortcuts.ctrlDDesc') }}</span>
          </div>
        </div>

        <div class="flex items-center justify-between">
          <button
            @click="skip"
            class="text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
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
                  idx === currentStep ? 'bg-primary-600' : idx < currentStep ? 'bg-primary-300' : 'bg-gray-300 dark:bg-gray-600'
                ]"
              />
            </div>
            <button
              v-if="currentStepData.action && currentStep > 0"
              @click="goToStep"
              class="px-4 py-2 bg-primary-100 text-primary-700 dark:bg-primary-900/30 dark:text-primary-400 rounded-lg hover:bg-primary-200 dark:hover:bg-primary-800/50 text-sm font-medium"
            >
              {{ t('onboarding.startExperience') }}
            </button>
            <button
              @click="next"
              class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium"
            >
              {{ isLastStep ? t('onboarding.startUsing') : t('onboarding.next') }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
