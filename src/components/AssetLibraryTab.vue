<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useAssetLibrary } from '@/composables/useAssetLibrary'
import { usePluginStore } from '@/stores'
import { ref, type Ref } from 'vue'
import type { Plugin } from '@/types'
import OneClickInstallDialog from '@/components/OneClickInstallDialog.vue'

const props = defineProps<{
  activeTab: 'repository' | 'bindings' | 'assetLibrary'
  loadPlugins: (force?: boolean) => Promise<void>
  showPostImportGuide: (pluginName: string, plugin?: Plugin) => Promise<void>
}>()

const { t } = useI18n()
const pluginStore = usePluginStore()
const activeTabRef = ref(props.activeTab) as Ref<'repository' | 'bindings' | 'assetLibrary'>

const {
  assetSearchQuery,
  assetSearchResults,
  isSearchingAssets,
  assetCategories,
  assetFilterType,
  assetFilterCategory,
  assetFilterGodotVersion,
  assetFilterSupport,
  assetSortBy,
  assetCurrentPage,
  assetTotalPages,
  assetTotalItems,
  selectedAssetIds,
  assetDetail,
  showAssetDetailDialog,
  importedAssetIds,
  hasSearched,
  recommendations,
  isLoadingRecommendations,
  projects,
  showOneClickDialog,
  oneClickPluginName,
  oneClickResult,
  isOneClickInstalling,
  oneClickStage,
  oneClickMessage,
  searchAssets,
  assetPrevPage,
  assetNextPage,
  toggleAssetSelection,
  importAsset,
  batchImportAssets,
  openAssetDetail,
  openPreviewLink,
  oneClickInstall,
  closeOneClickDialog,
} = useAssetLibrary({
  activeTab: activeTabRef,
  loadPlugins: props.loadPlugins,
  showPostImportGuide: props.showPostImportGuide,
})

// ─── Install dropdown state ───
const openDropdownId = ref<string | null>(null)

const toggleDropdown = (assetId: string) => {
  if (openDropdownId.value === assetId) {
    openDropdownId.value = null
  } else {
    openDropdownId.value = assetId
  }
}

const closeDropdown = () => {
  openDropdownId.value = null
}

const handleOneClickInstall = (assetId: string, assetTitle: string, projectId: string, autoApply: boolean) => {
  closeDropdown()
  oneClickInstall(Number(assetId), assetTitle, projectId, autoApply)
}

// ─── Compatibility badge ───
const getCompatibilityBadge = (godotVersion: string) => {
  const gv = godotVersion?.toLowerCase() || ''
  if (gv.includes('4.') || gv.includes('4.x')) {
    return { text: 'Godot 4', class: 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400' }
  } else if (gv.includes('3.') || gv.includes('3.x')) {
    return { text: 'Godot 3', class: 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400' }
  }
  return null
}

// ─── Rating display ───
const getRatingStars = (rating: string) => {
  const r = parseFloat(rating) || 0
  const full = Math.floor(r)
  const half = r - full >= 0.5
  const empty = 5 - full - (half ? 1 : 0)
  return { full, half, empty, value: r }
}

// ─── Support level badge ───
const getSupportBadge = (level: string) => {
  switch (level) {
    case 'official': return { text: t('assetLibrary.supportOfficial'), class: 'bg-blue-100 text-blue-800 dark:bg-surface-hover dark:text-brand-primary' }
    case 'featured': return { text: t('assetLibrary.supportFeatured'), class: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400' }
    case 'community': return { text: t('assetLibrary.supportCommunity'), class: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' }
    case 'testing': return { text: t('assetLibrary.supportTesting'), class: 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400' }
    default: return null
  }
}

// ─── Recommendation compatibility badge ───
const getRecCompatibilityBadge = (godotVersion: string) => {
  return getCompatibilityBadge(godotVersion)
}

const getRecRatingStars = (rating: number) => {
  const full = Math.floor(rating)
  const half = rating - full >= 0.5
  const empty = 5 - full - (half ? 1 : 0)
  return { full, half, empty }
}
</script>

<template>
  <div class="space-y-3">
    <!-- Recommendations Section -->
    <div v-if="recommendations.length > 0 && !hasSearched" class="mb-4">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3 flex items-center gap-2">
        <svg class="w-4 h-4 text-amber-500" fill="currentColor" viewBox="0 0 20 20">
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
        </svg>
        {{ t('assetLibrary.recommendations') }}
      </h2>
      <div class="flex gap-3 overflow-x-auto pb-2">
        <div
          v-for="rec in recommendations.slice(0, 6)"
          :key="rec.plugin.asset_id"
          class="bg-white dark:bg-surface-card rounded-md border border-gray-200 dark:border-surface-border transition-all duration-200 p-3 min-w-[220px] max-w-[260px] flex-shrink-0"
        >
          <div class="flex items-center gap-2 mb-2">
            <div class="w-9 h-9 rounded bg-gray-100 dark:bg-surface-layer flex items-center justify-center flex-shrink-0 overflow-hidden">
              <img
                v-if="rec.plugin.icon_url"
                :src="rec.plugin.icon_url"
                :alt="rec.plugin.name"
                class="w-8 h-8 object-contain"
                @error="($event.target as HTMLImageElement).style.display = 'none'"
              />
              <svg v-else class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate">{{ rec.plugin.name }}</div>
              <div class="text-xs text-gray-500 dark:text-content-muted truncate">{{ rec.plugin.author }}</div>
            </div>
          </div>

          <div class="flex items-center gap-2 mb-2">
            <div class="flex items-center gap-0.5" :title="`${rec.plugin.rating.toFixed(1)} / 5`">
              <svg v-for="n in getRecRatingStars(rec.plugin.rating).full" :key="n" class="w-3 h-3 text-amber-400" fill="currentColor" viewBox="0 0 20 20">
                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
              </svg>
              <svg v-for="n in getRecRatingStars(rec.plugin.rating).empty" :key="'e'+n" class="w-3 h-3 text-gray-300 dark:text-gray-600" fill="currentColor" viewBox="0 0 20 20">
                <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
              </svg>
              <span class="text-[10px] text-gray-500 dark:text-content-muted ml-0.5">{{ rec.plugin.rating.toFixed(1) }}</span>
            </div>
            <span v-if="getRecCompatibilityBadge(rec.plugin.godot_version)" :class="['px-1 py-0.5 rounded text-[10px] font-medium', getRecCompatibilityBadge(rec.plugin.godot_version)!.class]">
              {{ getRecCompatibilityBadge(rec.plugin.godot_version)!.text }}
            </span>
          </div>

          <div class="flex items-center gap-2">
            <button
              v-if="!importedAssetIds.has(String(rec.plugin.asset_id))"
              @click="importAsset(String(rec.plugin.asset_id), rec.plugin.name)"
              :disabled="pluginStore.isImporting === String(rec.plugin.asset_id)"
              class="px-2 py-1 bg-primary-600 hover:bg-primary-700 text-white rounded text-xs font-medium transition-colors disabled:opacity-50"
            >
              {{ pluginStore.isImporting === String(rec.plugin.asset_id) ? t('assetLibrary.importing') : t('assetLibrary.import') }}
            </button>
            <span v-else class="text-xs text-green-600 dark:text-green-400 font-medium">✓ {{ t('assetLibrary.alreadyImported') }}</span>

            <div v-if="projects.length > 0 && !importedAssetIds.has(String(rec.plugin.asset_id))" class="relative">
              <button
                @click="toggleDropdown(String(rec.plugin.asset_id))"
                class="px-2 py-1 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card rounded text-xs font-medium text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors"
              >
                {{ t('assetLibrary.oneClickInstall') }}
              </button>
              <div
                v-if="openDropdownId === String(rec.plugin.asset_id)"
                class="absolute right-0 top-full mt-1 w-48 bg-white dark:bg-surface-card rounded border border-gray-200 dark:border-surface-border z-50 py-1 max-h-48 overflow-y-auto"
              >
                <div class="px-3 py-1.5 text-xs text-gray-500 dark:text-content-muted border-b border-gray-100 dark:border-surface-border">
                  {{ t('assetLibrary.selectProject') }}
                </div>
                <button
                  v-for="project in projects"
                  :key="project.project_id"
                  @click="handleOneClickInstall(String(rec.plugin.asset_id), rec.plugin.name, project.project_id, false)"
                  class="w-full text-left px-3 py-2 text-xs hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors"
                >
                  <div class="font-medium text-gray-900 dark:text-content-primary">{{ project.name }}</div>
                  <div class="text-gray-500 dark:text-content-muted text-[10px]">Godot {{ project.godot_version }}</div>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-if="isLoadingRecommendations" class="text-center py-3">
        <div class="animate-spin w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full mx-auto"></div>
      </div>
    </div>

    <!-- Search Bar -->
    <div class="flex gap-2 mb-3">
      <input
        v-model="assetSearchQuery"
        type="text"
        :placeholder="t('assetLibrary.searchPlaceholder')"
        class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
        @input="searchAssets()"
        @keyup.enter="searchAssets(true)"
      />
      <button
        @click="searchAssets(true)"
        :disabled="isSearchingAssets"
        class="btn-primary disabled:opacity-50 text-sm"
      >
        {{ isSearchingAssets ? t('assetLibrary.searching') : t('assetLibrary.search') }}
      </button>
    </div>

    <div v-if="assetSearchResults.length === 0 && !isSearchingAssets && !hasSearched" class="text-center py-2">
      <p class="text-xs text-gray-400 dark:text-content-muted">{{ t('assetLibrary.initialHint') }}</p>
    </div>

    <!-- Filters -->
    <div class="flex flex-wrap gap-2 mb-3">
      <select v-model="assetFilterType" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="any">{{ t('assetLibrary.typeAny') }}</option>
        <option value="addon">{{ t('assetLibrary.typeAddon') }}</option>
        <option value="project">{{ t('assetLibrary.typeProject') }}</option>
      </select>
      <select v-model="assetFilterCategory" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="">{{ t('assetLibrary.categoryAll') }}</option>
        <option v-for="cat in assetCategories" :key="cat.id" :value="cat.id">{{ cat.name }}</option>
      </select>
      <select v-model="assetFilterGodotVersion" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="any">{{ t('assetLibrary.godotVersionAny') }}</option>
        <option value="4.0">{{ t('assetLibrary.godot4x') }}</option>
        <option value="3.0">{{ t('assetLibrary.godot3x') }}</option>
      </select>
      <select v-model="assetFilterSupport" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="">{{ t('assetLibrary.supportAll') }}</option>
        <option value="official">{{ t('assetLibrary.supportOfficial') }}</option>
        <option value="featured">{{ t('assetLibrary.supportFeatured') }}</option>
        <option value="community">{{ t('assetLibrary.supportCommunity') }}</option>
        <option value="testing">{{ t('assetLibrary.supportTesting') }}</option>
      </select>
      <select v-model="assetSortBy" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="updated">{{ t('assetLibrary.sortUpdated') }}</option>
        <option value="rating">{{ t('assetLibrary.sortRating') }}</option>
        <option value="name">{{ t('assetLibrary.sortName') }}</option>
        <option value="cost">{{ t('assetLibrary.sortCost') }}</option>
      </select>
    </div>

    <!-- Batch Import Bar -->
    <div v-if="selectedAssetIds.size > 0" class="bg-primary-50 dark:bg-surface-hover border border-primary-200 dark:border-surface-border rounded p-2 mb-3 flex items-center justify-between">
      <span class="text-xs font-medium text-primary-700 dark:text-content-secondary">{{ t('assetLibrary.selectedCount', { count: selectedAssetIds.size }) }}</span>
      <button
        @click="batchImportAssets"
        :disabled="!!pluginStore.isImporting"
        class="px-3 py-1 bg-primary-600 text-white text-xs rounded hover:bg-primary-700 disabled:opacity-50"
      >
        {{ t('assetLibrary.batchImport') }} ({{ selectedAssetIds.size }})
      </button>
    </div>

    <!-- Import Progress -->
    <div v-if="pluginStore.importProgress && pluginStore.isImporting" class="mb-3">
      <div class="flex items-center justify-between text-xs text-gray-600 dark:text-content-secondary mb-1">
        <span>{{ pluginStore.importProgress.message }}</span>
        <span>{{ Math.round(pluginStore.importProgress.progress * 100) }}%</span>
      </div>
      <div class="w-full bg-gray-200 dark:bg-surface-hover rounded-full h-2">
        <div
          class="bg-primary-600 h-2 rounded-full transition-all duration-300"
          :style="{ width: `${pluginStore.importProgress.progress * 100}%` }"
        ></div>
      </div>
    </div>

    <!-- Search Results -->
    <div class="space-y-2">
      <div v-if="assetSearchResults.length === 0 && !isSearchingAssets && hasSearched" class="text-center py-8 text-gray-500 dark:text-content-secondary">
        {{ t('assetLibrary.noResults') }}
      </div>
      <div v-if="isSearchingAssets" class="flex justify-center py-8">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
      <div
        v-for="asset in assetSearchResults"
        :key="asset.asset_id"
        :class="[
          'bg-white dark:bg-surface-card rounded-md border border-border p-3 transition-colors',
          selectedAssetIds.has(asset.asset_id) ? 'ring-2 ring-primary-500' : ''
        ]"
      >
        <div class="flex items-center gap-3">
          <input
            type="checkbox"
            :checked="selectedAssetIds.has(asset.asset_id)"
            @change="toggleAssetSelection(asset.asset_id)"
            class="w-4 h-4 text-primary-600 rounded flex-shrink-0 cursor-pointer"
          />
          <img
            v-if="asset.icon_url"
            :src="asset.icon_url"
            :alt="asset.title"
            class="w-10 h-10 rounded object-cover flex-shrink-0"
            loading="lazy"
            @error="($event.target as HTMLImageElement).style.display = 'none'"
          />
          <div v-else class="w-10 h-10 rounded bg-gray-200 dark:bg-surface-layer flex items-center justify-center flex-shrink-0">
            <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
            </svg>
          </div>
          <div class="flex-1 min-w-0 cursor-pointer" @click="openAssetDetail(asset.asset_id)">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="font-medium text-gray-900 dark:text-content-primary text-sm truncate">{{ asset.title }}</span>
              <span v-if="getSupportBadge(asset.support_level)" :class="['px-1.5 py-0.5 rounded text-[10px] font-medium', getSupportBadge(asset.support_level)!.class]">{{ getSupportBadge(asset.support_level)!.text }}</span>
              <span v-if="getCompatibilityBadge(asset.godot_version)" :class="['px-1.5 py-0.5 rounded text-[10px] font-medium', getCompatibilityBadge(asset.godot_version)!.class]">{{ getCompatibilityBadge(asset.godot_version)!.text }}</span>
            </div>
            <div class="flex items-center gap-2 mt-0.5 flex-wrap">
              <span class="text-xs text-gray-500 dark:text-content-secondary">{{ asset.author }} · {{ asset.category }} · {{ asset.cost }}</span>
              <div v-if="asset.rating && parseFloat(asset.rating) > 0" class="flex items-center gap-0.5">
                <svg v-for="n in getRatingStars(asset.rating).full" :key="n" class="w-3 h-3 text-amber-400" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                </svg>
                <svg v-for="n in getRatingStars(asset.rating).empty" :key="'e'+n" class="w-3 h-3 text-gray-300 dark:text-gray-600" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                </svg>
                <span class="text-[10px] text-gray-500 dark:text-content-muted ml-0.5">{{ getRatingStars(asset.rating).value.toFixed(1) }}</span>
              </div>
            </div>
          </div>

          <!-- Import Actions -->
          <div v-if="!importedAssetIds.has(asset.asset_id)" class="flex items-center gap-1 flex-shrink-0">
            <button
              @click="importAsset(asset.asset_id, asset.title)"
              :disabled="pluginStore.isImporting === asset.asset_id"
              class="btn-primary disabled:opacity-50 text-xs px-3 py-1.5"
            >
              {{ pluginStore.isImporting === asset.asset_id ? t('assetLibrary.importing') : t('assetLibrary.import') }}
            </button>
            <div v-if="projects.length > 0" class="relative">
              <button
                @click="toggleDropdown(asset.asset_id)"
                :disabled="pluginStore.isImporting === asset.asset_id"
                class="px-1.5 py-1.5 border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-card rounded text-xs text-gray-600 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors disabled:opacity-50"
                :title="t('assetLibrary.oneClickInstallToProject')"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                </svg>
              </button>
              <div
                v-if="openDropdownId === asset.asset_id"
                class="absolute right-0 top-full mt-1 w-48 bg-white dark:bg-surface-card rounded border border-gray-200 dark:border-surface-border z-50 py-1 max-h-48 overflow-y-auto"
              >
                <div class="px-3 py-1.5 text-xs text-gray-500 dark:text-content-muted border-b border-gray-100 dark:border-surface-border">
                  {{ t('assetLibrary.selectProject') }}
                </div>
                <button
                  v-for="project in projects"
                  :key="project.project_id"
                  @click="handleOneClickInstall(asset.asset_id, asset.title, project.project_id, false)"
                  class="w-full text-left px-3 py-2 text-xs hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors"
                >
                  <div class="font-medium text-gray-900 dark:text-content-primary">{{ project.name }}</div>
                  <div class="text-gray-500 dark:text-content-muted text-[10px]">Godot {{ project.godot_version }}</div>
                </button>
              </div>
            </div>
          </div>
          <span v-else class="text-xs px-3 py-1.5 text-green-600 dark:text-green-400 flex-shrink-0 font-medium">✓ {{ t('assetLibrary.alreadyImported') }}</span>
        </div>
      </div>
    </div>

    <!-- Pagination -->
    <div v-if="assetTotalPages > 0" class="flex items-center justify-between mt-4 pt-3 border-t border-gray-200 dark:border-surface-border">
      <span class="text-xs text-gray-500 dark:text-content-secondary">
        {{ t('assetLibrary.totalItems', { count: assetTotalItems }) }}
      </span>
      <div class="flex items-center gap-2">
        <button
          @click="assetPrevPage"
          :disabled="assetCurrentPage === 0"
          class="px-3 py-1 text-xs border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-card disabled:opacity-50"
        >
          {{ t('assetLibrary.prevPage') }}
        </button>
        <span class="text-xs text-gray-600 dark:text-content-secondary">
          {{ t('assetLibrary.page', { current: assetCurrentPage + 1, total: assetTotalPages }) }}
        </span>
        <button
          @click="assetNextPage"
          :disabled="assetCurrentPage >= assetTotalPages - 1"
          class="px-3 py-1 text-xs border border-gray-300 dark:border-surface-border rounded bg-white dark:bg-surface-layer text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-card disabled:opacity-50"
        >
          {{ t('assetLibrary.nextPage') }}
        </button>
      </div>
    </div>

    <!-- Asset Detail Dialog -->
    <Teleport to="body">
      <div v-if="showAssetDetailDialog && assetDetail" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showAssetDetailDialog = false; assetDetail = null">
        <div class="dialog-container w-full max-w-lg max-h-[80vh] flex flex-col" @click.stop>
          <div class="flex justify-between items-center mb-3">
            <h3 class="dialog-title mb-0">{{ assetDetail.title }}</h3>
            <button @click="showAssetDetailDialog = false; assetDetail = null" class="text-gray-500 dark:text-content-secondary hover:text-gray-700 dark:hover:text-content-primary">
              <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div class="flex items-center gap-3 mb-4">
            <img
              v-if="assetDetail.icon_url"
              :src="assetDetail.icon_url"
              :alt="assetDetail.title"
              class="w-12 h-12 rounded object-cover"
            />
            <div>
              <div class="text-sm text-gray-600 dark:text-content-secondary">{{ t('assetLibrary.author') }}: {{ assetDetail.author }}</div>
              <div class="text-sm text-gray-600 dark:text-content-secondary">{{ t('assetLibrary.license') }}: {{ assetDetail.cost }}</div>
              <div class="text-sm text-gray-600 dark:text-content-secondary">{{ t('assetLibrary.rating') }}: {{ assetDetail.rating }}/5</div>
            </div>
          </div>

          <div v-if="assetDetail.previews && assetDetail.previews.length > 0" class="mb-4">
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('assetLibrary.previews') }}</h4>
            <div class="flex gap-2 overflow-x-auto pb-2">
              <img
                v-for="preview in assetDetail.previews.filter(p => p.type === 'image')"
                :key="preview.preview_id"
                :src="preview.thumbnail"
                class="h-20 rounded object-cover flex-shrink-0 cursor-pointer hover:opacity-80"
                loading="lazy"
                @click="openPreviewLink(preview.link)"
              />
            </div>
          </div>

          <div class="flex-1 overflow-y-auto mb-4">
            <h4 class="text-sm font-medium text-gray-700 dark:text-content-primary mb-2">{{ t('assetLibrary.description') }}</h4>
            <p class="text-sm text-gray-600 dark:text-content-secondary whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded p-3">
              {{ assetDetail.description || t('assetLibrary.noDescription') }}
            </p>
          </div>

          <div class="flex items-center gap-2">
            <button
              @click="importAsset(assetDetail.asset_id, assetDetail.title); showAssetDetailDialog = false; assetDetail = null"
              :disabled="pluginStore.isImporting === assetDetail.asset_id"
              class="btn-primary disabled:opacity-50 text-sm"
            >
              {{ pluginStore.isImporting === assetDetail.asset_id ? t('assetLibrary.importing') : t('assetLibrary.import') }}
            </button>
            <a
              v-if="assetDetail.browse_url"
              :href="assetDetail.browse_url"
              target="_blank"
              class="px-4 py-2 border border-gray-300 dark:border-surface-border rounded text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-card"
            >
              {{ t('assetLibrary.detail') }}
            </a>
            <div class="flex-1"></div>
            <button
              @click="showAssetDetailDialog = false; assetDetail = null"
              class="btn-secondary text-sm"
            >
              {{ t('common.close') }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

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
