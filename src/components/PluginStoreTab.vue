<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { usePluginStoreComposable } from '@/composables/usePluginStore'
import StorePluginCard from '@/components/StorePluginCard.vue'
import OneClickInstallDialog from '@/components/OneClickInstallDialog.vue'
import type { Project, OneClickInstallResult } from '@/types'

const props = defineProps<{
  loadPlugins: (force?: boolean) => Promise<void>
}>()

const { t } = useI18n()

const {
  searchQuery,
  searchResults,
  isSearching,
  categories,
  selectedCategory,
  sortBy,
  godotVersionFilter,
  currentPage,
  totalResults,
  hasMore,
  hasSearched,
  recommendations,
  isLoadingRecommendations,
  totalPages,
  doSearch,
  prevPage,
  nextPage,
  simpleImport,
  initStore,
} = usePluginStoreComposable()

const projects = ref<Project[]>([])
const showOneClickDialog = ref(false)
const oneClickPluginName = ref('')
const oneClickResult = ref<OneClickInstallResult | null>(null)
const isOneClickInstalling = ref(false)
const oneClickStage = ref<'downloading' | 'importing' | 'binding' | 'applying' | 'complete' | 'error' | null>(null)
const oneClickMessage = ref('')

const importingAssetId = ref<string | null>(null)

onMounted(async () => {
  try {
    projects.value = await api.getProjects()
  } catch (e) {
    console.error('Failed to load projects:', e)
  }
  await initStore()
})

const handleInstall = async (assetId: number, assetName: string) => {
  importingAssetId.value = String(assetId)
  try {
    await simpleImport(assetId, assetName)
  } finally {
    importingAssetId.value = null
  }
}

const handleOneClickInstall = async (assetId: number, assetName: string, projectId: string, autoApply: boolean) => {
  showOneClickDialog.value = true
  oneClickPluginName.value = assetName
  oneClickResult.value = null
  oneClickStage.value = 'downloading'
  oneClickMessage.value = t('pluginStore.stepDownloading')
  isOneClickInstalling.value = true

  try {
    oneClickStage.value = 'importing'
    oneClickMessage.value = t('pluginStore.stepImporting')

    const result = await api.oneClickInstallPlugin(assetId, projectId, autoApply)
    oneClickResult.value = result

    if (result.success) {
      oneClickStage.value = 'complete'
      oneClickMessage.value = t('pluginStore.installSuccess')
      if (result.binding_created) {
        oneClickStage.value = 'binding'
        await new Promise(r => setTimeout(r, 300))
        oneClickStage.value = 'applying'
        await new Promise(r => setTimeout(r, 300))
        oneClickStage.value = 'complete'
      }
      await props.loadPlugins(true)
      await doSearch(true)
    } else {
      oneClickStage.value = 'error'
      oneClickMessage.value = result.errors.join('; ')
    }
  } catch (error) {
    oneClickStage.value = 'error'
    oneClickMessage.value = String(error)
    oneClickResult.value = {
      success: false,
      plugin_id: null,
      binding_created: false,
      changes_applied: false,
      errors: [String(error)],
    }
  } finally {
    isOneClickInstalling.value = false
  }
}

const closeOneClickDialog = () => {
  if (!isOneClickInstalling.value) {
    showOneClickDialog.value = false
  }
}

const handleCategoryChange = () => {
  currentPage.value = 1
  doSearch(true)
}

const handleSortChange = () => {
  currentPage.value = 1
  doSearch(true)
}

const handleGodotVersionChange = () => {
  currentPage.value = 1
  doSearch(true)
}
</script>

<template>
  <div class="space-y-4">
    <!-- Recommendations Section -->
    <div v-if="recommendations.length > 0 && !hasSearched" class="mb-6">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3 flex items-center gap-2">
        <svg class="w-4 h-4 text-amber-500" fill="currentColor" viewBox="0 0 20 20">
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
        </svg>
        {{ t('pluginStore.recommendations') }}
      </h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        <StorePluginCard
          v-for="rec in recommendations"
          :key="rec.plugin.asset_id"
          :plugin="rec.plugin"
          :projects="projects"
          :is-installing="importingAssetId === String(rec.plugin.asset_id)"
          @install="handleInstall"
          @one-click-install="handleOneClickInstall"
        />
      </div>
      <div v-if="isLoadingRecommendations" class="text-center py-4">
        <div class="animate-spin w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full mx-auto"></div>
      </div>
    </div>

    <!-- Search Bar -->
    <div class="flex gap-2">
      <div class="flex-1 relative">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="t('pluginStore.searchPlaceholder')"
          class="w-full pl-10 pr-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          @input="doSearch()"
          @keyup.enter="doSearch(true)"
        />
      </div>
      <button
        @click="doSearch(true)"
        :disabled="isSearching"
        class="btn-primary disabled:opacity-50 text-sm px-4"
      >
        {{ isSearching ? t('assetLibrary.searching') : t('assetLibrary.search') }}
      </button>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap gap-2">
      <select
        v-model="selectedCategory"
        @change="handleCategoryChange"
        class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs"
      >
        <option value="">{{ t('pluginStore.allCategories') }}</option>
        <option v-for="cat in categories" :key="cat.id" :value="cat.id">
          {{ cat.icon }} {{ cat.name }}
        </option>
      </select>

      <select
        v-model="sortBy"
        @change="handleSortChange"
        class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs"
      >
        <option value="updated">{{ t('pluginStore.sortUpdated') }}</option>
        <option value="rating">{{ t('pluginStore.sortRating') }}</option>
        <option value="downloads">{{ t('pluginStore.sortDownloads') }}</option>
        <option value="name">{{ t('pluginStore.sortName') }}</option>
      </select>

      <select
        v-model="godotVersionFilter"
        @change="handleGodotVersionChange"
        class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs"
      >
        <option value="">{{ t('pluginStore.allVersions') }}</option>
        <option value="4">{{ t('pluginStore.godot4') }}</option>
        <option value="4.2">Godot 4.2+</option>
        <option value="4.3">Godot 4.3+</option>
        <option value="3">{{ t('pluginStore.godot3') }}</option>
      </select>

      <span v-if="totalResults > 0" class="text-xs text-gray-500 dark:text-content-muted self-center ml-auto">
        {{ t('pluginStore.totalResults', { count: totalResults }) }}
      </span>
    </div>

    <!-- Search Results -->
    <div v-if="isSearching && searchResults.length === 0" class="text-center py-12">
      <div class="animate-spin w-8 h-8 border-2 border-primary-600 border-t-transparent rounded-full mx-auto mb-3"></div>
      <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('pluginStore.searching') }}</p>
    </div>

    <div v-else-if="searchResults.length === 0 && hasSearched" class="text-center py-12">
      <svg class="w-12 h-12 text-gray-300 dark:text-content-muted mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
      </svg>
      <p class="text-sm text-gray-500 dark:text-content-muted">{{ t('pluginStore.noResults') }}</p>
    </div>

    <div v-else-if="searchResults.length === 0 && !hasSearched" class="text-center py-8">
      <p class="text-xs text-gray-400 dark:text-content-muted">{{ t('pluginStore.searchHint') }}</p>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      <StorePluginCard
        v-for="plugin in searchResults"
        :key="plugin.asset_id"
        :plugin="plugin"
        :projects="projects"
        :is-installing="importingAssetId === String(plugin.asset_id)"
        @install="handleInstall"
        @one-click-install="handleOneClickInstall"
      />
    </div>

    <!-- Pagination -->
    <div v-if="totalPages > 1" class="flex items-center justify-center gap-3 pt-4">
      <button
        @click="prevPage"
        :disabled="currentPage <= 1"
        class="px-3 py-1.5 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card rounded-lg text-xs text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors disabled:opacity-50"
      >
        {{ t('common.prev') }}
      </button>
      <span class="text-xs text-gray-500 dark:text-content-muted">
        {{ currentPage }} / {{ totalPages }}
      </span>
      <button
        @click="nextPage"
        :disabled="!hasMore"
        class="px-3 py-1.5 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card rounded-lg text-xs text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors disabled:opacity-50"
      >
        {{ t('common.next') }}
      </button>
    </div>

    <!-- One-Click Install Dialog -->
    <OneClickInstallDialog
      :visible="showOneClickDialog"
      :plugin-name="oneClickPluginName"
      :result="oneClickResult"
      :is-installing="isOneClickInstalling"
      :current-stage="oneClickStage"
      :stage-message="oneClickMessage"
      @close="closeOneClickDialog"
    />
  </div>
</template>
