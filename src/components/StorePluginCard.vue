<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { StorePlugin, Project } from '@/types'

const props = defineProps<{
  plugin: StorePlugin
  projects: Project[]
  isInstalling?: boolean
}>()

const emit = defineEmits<{
  (e: 'install', assetId: number, assetName: string): void
  (e: 'oneClickInstall', assetId: number, assetName: string, projectId: string, autoApply: boolean): void
}>()

const { t } = useI18n()

const showProjectDropdown = ref(false)

const supportLevelBadge = computed(() => {
  switch (props.plugin.support_level) {
    case 'official': return { text: '官方', class: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400' }
    case 'featured': return { text: '精选', class: 'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400' }
    case 'community': return { text: '社区', class: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' }
    case 'testing': return { text: '测试', class: 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400' }
    default: return { text: '社区', class: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' }
  }
})

const compatibilityBadge = computed(() => {
  const gv = props.plugin.godot_version?.toLowerCase() || ''
  if (gv.includes('4.') || gv.includes('4.x')) {
    return { text: 'Godot 4', class: 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400' }
  } else if (gv.includes('3.') || gv.includes('3.x')) {
    return { text: 'Godot 3', class: 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400' }
  }
  return null
})

const ratingStars = computed(() => {
  const full = Math.floor(props.plugin.rating)
  const half = props.plugin.rating - full >= 0.5
  const empty = 5 - full - (half ? 1 : 0)
  return { full, half, empty }
})

const formatDownloadCount = (count: number) => {
  if (count >= 10000) return (count / 1000).toFixed(1) + 'K'
  if (count >= 1000) return (count / 1000).toFixed(1) + 'K'
  return String(count)
}

const handleInstall = () => {
  emit('install', props.plugin.asset_id, props.plugin.name)
}

const handleOneClickInstall = (projectId: string, autoApply: boolean) => {
  showProjectDropdown.value = false
  emit('oneClickInstall', props.plugin.asset_id, props.plugin.name, projectId, autoApply)
}

const toggleProjectDropdown = () => {
  showProjectDropdown.value = !showProjectDropdown.value
}
</script>

<template>
  <div class="bg-white dark:bg-surface-card rounded-xl border border-gray-200 dark:border-surface-border hover:shadow-md transition-all duration-200 overflow-hidden group">
    <div class="p-4">
      <div class="flex items-start gap-3">
        <div class="w-12 h-12 rounded-lg bg-gray-100 dark:bg-surface-layer flex items-center justify-center flex-shrink-0 overflow-hidden">
          <img
            v-if="plugin.icon_url"
            :src="plugin.icon_url"
            :alt="plugin.name"
            class="w-10 h-10 object-contain"
            @error="($event.target as HTMLImageElement).style.display = 'none'"
          />
          <svg v-else class="w-6 h-6 text-gray-400 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        </div>

        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 mb-1">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary truncate">{{ plugin.name }}</h3>
            <span :class="['px-1.5 py-0.5 rounded text-[10px] font-medium', supportLevelBadge.class]">{{ supportLevelBadge.text }}</span>
            <span v-if="compatibilityBadge" :class="['px-1.5 py-0.5 rounded text-[10px] font-medium', compatibilityBadge.class]">{{ compatibilityBadge.text }}</span>
          </div>

          <p class="text-xs text-gray-500 dark:text-content-muted mb-2 truncate">{{ plugin.author }}</p>

          <p class="text-xs text-gray-600 dark:text-content-secondary line-clamp-2 mb-2" style="display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;">
            {{ plugin.description }}
          </p>

          <div class="flex items-center gap-3 text-xs text-gray-500 dark:text-content-muted">
            <div class="flex items-center gap-0.5" :title="`${plugin.rating.toFixed(1)} / 5`">
              <svg v-for="n in ratingStars.full" :key="n" class="w-3.5 h-3.5 text-amber-400" fill="currentColor" viewBox="0 0 20 20">
                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
              </svg>
              <template v-if="ratingStars.half">
                <svg class="w-3.5 h-3.5 text-amber-400" fill="currentColor" viewBox="0 0 20 20">
                  <defs><linearGradient id="half"><stop offset="50%" stop-color="currentColor"/><stop offset="50%" stop-color="#D1D5DB"/></linearGradient></defs>
                  <path fill="url(#half)" d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                </svg>
              </template>
              <svg v-for="n in ratingStars.empty" :key="'e'+n" class="w-3.5 h-3.5 text-gray-300 dark:text-gray-600" fill="currentColor" viewBox="0 0 20 20">
                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
              </svg>
              <span class="ml-0.5">{{ plugin.rating.toFixed(1) }}</span>
            </div>

            <span v-if="plugin.download_count > 0" class="flex items-center gap-1">
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
              </svg>
              {{ formatDownloadCount(plugin.download_count) }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="plugin.tags.length > 0" class="flex flex-wrap gap-1 mt-2">
        <span
          v-for="tag in plugin.tags.slice(0, 3)"
          :key="tag"
          class="px-1.5 py-0.5 bg-gray-100 dark:bg-surface-layer rounded text-[10px] text-gray-600 dark:text-content-muted"
        >{{ tag }}</span>
        <span v-if="plugin.tags.length > 3" class="px-1.5 py-0.5 text-[10px] text-gray-400 dark:text-content-muted">+{{ plugin.tags.length - 3 }}</span>
      </div>
    </div>

    <div class="px-4 py-2.5 border-t border-gray-100 dark:border-surface-border bg-gray-50/50 dark:bg-surface-layer/50 flex items-center justify-between">
      <div v-if="plugin.is_installed" class="flex items-center gap-1.5 text-xs text-green-600 dark:text-green-400">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <span>{{ t('assetLibrary.imported') }}</span>
        <span v-if="plugin.installed_version" class="text-gray-400 dark:text-content-muted">v{{ plugin.installed_version }}</span>
      </div>

      <div v-else class="flex items-center gap-2">
        <button
          @click="handleInstall"
          :disabled="isInstalling"
          class="px-3 py-1.5 bg-primary-600 hover:bg-primary-700 text-white rounded-lg text-xs font-medium transition-colors disabled:opacity-50"
        >
          {{ isInstalling ? t('assetLibrary.importing') : t('assetLibrary.import') }}
        </button>

        <div class="relative" v-if="projects.length > 0">
          <button
            @click="toggleProjectDropdown"
            class="px-2 py-1.5 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card rounded-lg text-xs font-medium text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors flex items-center gap-1"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            {{ t('pluginStore.oneClickInstall') }}
          </button>

          <div
            v-if="showProjectDropdown"
            class="absolute right-0 bottom-full mb-1 w-56 bg-white dark:bg-surface-card rounded-lg shadow-lg border border-gray-200 dark:border-surface-border z-50 py-1 max-h-60 overflow-y-auto"
          >
            <div class="px-3 py-1.5 text-xs text-gray-500 dark:text-content-muted border-b border-gray-100 dark:border-surface-border">
              {{ t('pluginStore.selectProject') }}
            </div>
            <button
              v-for="project in projects"
              :key="project.project_id"
              @click="handleOneClickInstall(project.project_id, false)"
              class="w-full text-left px-3 py-2 text-xs hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors"
            >
              <div class="font-medium text-gray-900 dark:text-content-primary">{{ project.name }}</div>
              <div class="text-gray-500 dark:text-content-muted text-[10px]">Godot {{ project.godot_version }}</div>
            </button>
          </div>
        </div>
      </div>

      <a
        v-if="plugin.source_url"
        :href="plugin.source_url"
        target="_blank"
        class="text-gray-400 hover:text-gray-600 dark:text-content-muted dark:hover:text-content-secondary transition-colors"
        :title="t('pluginStore.viewSource')"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
        </svg>
      </a>
    </div>
  </div>
</template>
