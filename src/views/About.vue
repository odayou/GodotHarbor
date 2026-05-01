<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { api } from '@/api'
import { useToast } from '@/composables/useToast'

const { t } = useI18n()
const router = useRouter()
const toast = useToast()

const appVersion = ref('')
const activeTab = ref<'about' | 'credits' | 'sponsor'>('about')
const isCheckingUpdate = ref(false)

onMounted(async () => {
  try {
    appVersion.value = await api.getAppVersion()
  } catch {
    appVersion.value = '0.1.0'
  }
})

const copyVersion = async () => {
  try {
    await navigator.clipboard.writeText(`Godot Harbor v${appVersion.value}`)
    toast.success(t('about.versionCopied'))
  } catch {
    toast.error(t('settings.messages.copyFailed'))
  }
}

const checkForUpdates = async () => {
  isCheckingUpdate.value = true
  try {
    const result = await api.checkAllUpdates()
    const hasAppUpdate = !!result.app_update || !!result.hot_update
    const hasPluginUpdates = result.plugin_updates.length > 0
    const hasEngineUpdates = result.engine_updates.length > 0
    if (hasAppUpdate || hasPluginUpdates || hasEngineUpdates) {
      router.push('/updates')
    } else {
      toast.success(t('about.upToDate'))
    }
  } catch (error) {
    toast.error(t('about.checkUpdateFailed'))
  } finally {
    isCheckingUpdate.value = false
  }
}

const rustDeps = [
  { name: 'Tauri', version: '2.x', license: 'Apache-2.0 / MIT', url: 'https://tauri.app' },
  { name: 'Serde', version: '1.x', license: 'Apache-2.0 / MIT', url: 'https://github.com/serde-rs/serde' },
  { name: 'Tokio', version: '1.x', license: 'MIT', url: 'https://tokio.rs' },
  { name: 'UUID', version: '1.x', license: 'Apache-2.0 / MIT', url: 'https://github.com/uuid-rs/uuid' },
  { name: 'Chrono', version: '0.4', license: 'Apache-2.0 / MIT', url: 'https://github.com/chronotope/chrono' },
  { name: 'WalkDir', version: '2.x', license: 'Unlicense / MIT', url: 'https://github.com/BurntSushi/walkdir' },
  { name: 'Git2', version: '0.18', license: 'Apache-2.0 / MIT', url: 'https://github.com/rust-lang/git2-rs' },
  { name: 'Anyhow', version: '1.x', license: 'Apache-2.0 / MIT', url: 'https://github.com/dtolnay/anyhow' },
  { name: 'Reqwest', version: '0.12', license: 'Apache-2.0 / MIT', url: 'https://github.com/seanmonstar/reqwest' },
  { name: 'Zip', version: '2.x', license: 'MIT', url: 'https://github.com/zip-rs/zip2' },
]

const jsDeps = [
  { name: 'Vue.js', version: '3.x', license: 'MIT', url: 'https://vuejs.org' },
  { name: 'Vue Router', version: '4.x', license: 'MIT', url: 'https://router.vuejs.org' },
  { name: 'Pinia', version: '2.x', license: 'MIT', url: 'https://pinia.vuejs.org' },
  { name: 'Vite', version: '5.x', license: 'MIT', url: 'https://vitejs.dev' },
  { name: 'TypeScript', version: '5.x', license: 'Apache-2.0', url: 'https://www.typescriptlang.org' },
  { name: 'Tailwind CSS', version: '3.x', license: 'MIT', url: 'https://tailwindcss.com' },
]

const inspirationDeps = [
  { name: 'Godot Engine', desc: 'about.godotEngineDesc', url: 'https://godotengine.org' },
  { name: 'GodotEnv', desc: 'about.godotEnvDesc', url: 'https://github.com/nicholas-mikusic/godotenv' },
  { name: 'gd-plug', desc: 'about.gdplugDesc', url: 'https://github.com/imjp94/gd-plug' },
  { name: 'godam', desc: 'about.godamDesc', url: 'https://github.com/youyo/godam' },
]
</script>

<template>
  <div class="space-y-6">
    <div class="flex gap-2 border-b border-gray-200 dark:border-gray-700 pb-2">
      <button
        @click="activeTab = 'about'"
        :class="['px-4 py-2 text-sm font-medium rounded-t-lg transition-colors', activeTab === 'about' ? 'bg-primary-600 text-white' : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200']"
      >
        {{ t('about.tabs.about') }}
      </button>
      <button
        @click="activeTab = 'credits'"
        :class="['px-4 py-2 text-sm font-medium rounded-t-lg transition-colors', activeTab === 'credits' ? 'bg-primary-600 text-white' : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200']"
      >
        {{ t('about.tabs.credits') }}
      </button>
      <button
        @click="activeTab = 'sponsor'"
        :class="['px-4 py-2 text-sm font-medium rounded-t-lg transition-colors', activeTab === 'sponsor' ? 'bg-primary-600 text-white' : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200']"
      >
        {{ t('about.tabs.sponsor') }}
      </button>
    </div>

    <div v-if="activeTab === 'about'" class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-8 text-center">
        <div class="w-20 h-20 mx-auto mb-4 flex items-center justify-center">
          <img src="../assets/StoreLogo.png" alt="Godot Harbor Logo" class="w-full h-full object-contain" />
        </div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">Godot Harbor</h1>
        <div class="flex items-center justify-center gap-2 mt-2">
          <p class="text-gray-500 dark:text-gray-400">v{{ appVersion }}</p>
          <button
            @click="copyVersion"
            class="p-1 rounded text-gray-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
            :title="t('about.copyVersion')"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          </button>
        </div>
        <p class="text-gray-600 dark:text-gray-400 mt-4 max-w-xl mx-auto">
          {{ t('about.appDescription') }}
        </p>
        <button
          @click="checkForUpdates"
          :disabled="isCheckingUpdate"
          class="mt-4 px-4 py-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-600 transition-colors text-sm disabled:opacity-50"
        >
          <span v-if="isCheckingUpdate" class="flex items-center gap-2">
            <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            {{ t('about.checking') }}
          </span>
          <span v-else>{{ t('about.checkUpdates') }}</span>
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
          <div class="w-10 h-10 mx-auto mb-3 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <h3 class="font-semibold text-gray-900 dark:text-gray-100">{{ t('about.features.symlinkEngine.title') }}</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">{{ t('about.features.symlinkEngine.desc') }}</p>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
          <div class="w-10 h-10 mx-auto mb-3 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
          </div>
          <h3 class="font-semibold text-gray-900 dark:text-gray-100">{{ t('about.features.versionManagement.title') }}</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">{{ t('about.features.versionManagement.desc') }}</p>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
          <div class="w-10 h-10 mx-auto mb-3 bg-purple-100 dark:bg-purple-900/30 rounded-lg flex items-center justify-center">
            <svg class="w-6 h-6 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </div>
          <h3 class="font-semibold text-gray-900 dark:text-gray-100">{{ t('about.features.teamCollaboration.title') }}</h3>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-2">{{ t('about.features.teamCollaboration.desc') }}</p>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">{{ t('about.techStack') }}</h3>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">🦀</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('about.rust') }}</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('about.rustBackend') }}</p>
            </div>
          </div>
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">⚡</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('about.tauri') }}</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('about.tauriDesktop') }}</p>
            </div>
          </div>
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">💚</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('about.vue') }}</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('about.vueFrontend') }}</p>
            </div>
          </div>
          <div class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
            <span class="text-2xl">🎨</span>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ t('about.tailwind') }}</p>
              <p class="text-xs text-gray-500 dark:text-gray-400">{{ t('about.tailwindCSS') }}</p>
            </div>
          </div>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {{ t('about.openSource') }}
        </p>
        <div class="flex justify-center gap-4 mt-4">
          <a href="https://github.com/odayou/GodotHarbor" target="_blank" class="text-primary-600 dark:text-primary-400 hover:underline text-sm">
            {{ t('about.githubRepo') }}
          </a>
          <a href="https://github.com/odayou/GodotHarbor/issues" target="_blank" class="text-primary-600 dark:text-primary-400 hover:underline text-sm">
            {{ t('about.issueReport') }}
          </a>
          <a href="https://github.com/odayou/GodotHarbor/contribute" target="_blank" class="text-primary-600 dark:text-primary-400 hover:underline text-sm">
            {{ t('about.contribute') }}
          </a>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'credits'" class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">{{ t('about.creditsTitle') }}</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
          {{ t('about.creditsDesc') }}
        </p>

        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">{{ t('about.rustDeps') }}</h3>
        <div class="space-y-2 mb-6">
          <div
            v-for="dep in rustDeps"
            :key="dep.name"
            class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
          >
            <div class="flex items-center gap-3">
              <a :href="dep.url" target="_blank" class="text-sm font-medium text-primary-600 dark:text-primary-400 hover:underline">
                {{ dep.name }}
              </a>
              <span class="text-xs text-gray-500 dark:text-gray-400">v{{ dep.version }}</span>
            </div>
            <span class="text-xs text-gray-400 dark:text-gray-500">{{ dep.license }}</span>
          </div>
        </div>

        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">{{ t('about.jsDeps') }}</h3>
        <div class="space-y-2 mb-6">
          <div
            v-for="dep in jsDeps"
            :key="dep.name"
            class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
          >
            <div class="flex items-center gap-3">
              <a :href="dep.url" target="_blank" class="text-sm font-medium text-primary-600 dark:text-primary-400 hover:underline">
                {{ dep.name }}
              </a>
              <span class="text-xs text-gray-500 dark:text-gray-400">v{{ dep.version }}</span>
            </div>
            <span class="text-xs text-gray-400 dark:text-gray-500">{{ dep.license }}</span>
          </div>
        </div>

        <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100 mb-3">{{ t('about.inspiration') }}</h3>
        <div class="space-y-2">
          <div
            v-for="dep in inspirationDeps"
            :key="dep.name"
            class="p-3 bg-gray-50 dark:bg-gray-700 rounded-lg"
          >
            <a :href="dep.url" target="_blank" class="text-sm font-medium text-gray-900 dark:text-gray-100 hover:underline">
              {{ dep.name }}
            </a>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">{{ t(dep.desc) }}</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'sponsor'" class="space-y-6">
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-8 text-center">
        <div class="w-16 h-16 mx-auto mb-4 bg-pink-100 dark:bg-pink-900/30 rounded-full flex items-center justify-center">
          <svg class="w-8 h-8 text-pink-600 dark:text-pink-400" fill="currentColor" viewBox="0 0 24 24">
            <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
          </svg>
        </div>
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">{{ t('about.sponsorTitle') }}</h2>
        <p class="text-gray-600 dark:text-gray-400 max-w-md mx-auto mb-6">
          {{ t('about.sponsorDesc') }}
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-yellow-100 dark:bg-yellow-900/30 rounded-lg flex items-center justify-center">
              <span class="text-xl">☕</span>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('about.buyMeCoffee') }}</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            {{ t('about.coffeeDesc') }}
          </p>
          <a href="https://buymeacoffee.com" target="_blank" class="inline-block px-4 py-2 bg-yellow-500 text-white rounded-lg hover:bg-yellow-600 text-sm">
            {{ t('about.buyCoffee') }}
          </a>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-gray-900 dark:bg-gray-100 rounded-lg flex items-center justify-center">
              <svg class="w-5 h-5 text-white dark:text-gray-900" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('about.githubSponsors') }}</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            {{ t('about.githubSponsorsDesc') }}
          </p>
          <a href="https://github.com/sponsors" target="_blank" class="inline-block px-4 py-2 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 rounded-lg hover:bg-gray-800 dark:hover:bg-gray-200 text-sm">
            {{ t('about.becomeSponsor') }}
          </a>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
              <span class="text-xl">🌟</span>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('about.starGithub') }}</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            {{ t('about.starGithubDesc') }}
          </p>
          <a href="https://github.com/odayou/GodotHarbor" target="_blank" class="inline-block px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 text-sm">
            {{ t('about.giveStar') }}
          </a>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center gap-3 mb-4">
            <div class="w-10 h-10 bg-green-100 dark:bg-green-900/30 rounded-lg flex items-center justify-center">
              <span class="text-xl">🤝</span>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{{ t('about.contributeAction') }}</h3>
          </div>
          <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
            {{ t('about.contributeDesc') }}
          </p>
          <a href="https://github.com/odayou/GodotHarbor/contribute" target="_blank" class="inline-block px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 text-sm">
            {{ t('about.contributionGuide') }}
          </a>
        </div>
      </div>

      <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6 text-center">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {{ t('about.sponsorThanks') }}
        </p>
      </div>
    </div>
  </div>
</template>
