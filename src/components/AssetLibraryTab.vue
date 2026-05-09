<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useAssetLibrary } from '@/composables/useAssetLibrary'
import { usePluginStore } from '@/stores'
import { ref, type Ref } from 'vue'
import type { Plugin } from '@/types'

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
  searchAssets,
  assetPrevPage,
  assetNextPage,
  toggleAssetSelection,
  importAsset,
  batchImportAssets,
  openAssetDetail,
  openPreviewLink,
} = useAssetLibrary({
  activeTab: activeTabRef,
  loadPlugins: props.loadPlugins,
  showPostImportGuide: props.showPostImportGuide,
})
</script>

<template>
  <div class="space-y-4">
    <div class="flex gap-2 mb-3">
      <input
        v-model="assetSearchQuery"
        type="text"
        :placeholder="t('assetLibrary.searchPlaceholder')"
        class="flex-1 px-3 py-2 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-sm"
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

    <div class="flex flex-wrap gap-2 mb-3">
      <select v-model="assetFilterType" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="any">{{ t('assetLibrary.typeAny') }}</option>
        <option value="addon">{{ t('assetLibrary.typeAddon') }}</option>
        <option value="project">{{ t('assetLibrary.typeProject') }}</option>
      </select>
      <select v-model="assetFilterCategory" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="">{{ t('assetLibrary.categoryAll') }}</option>
        <option v-for="cat in assetCategories" :key="cat.id" :value="cat.id">{{ cat.name }}</option>
      </select>
      <select v-model="assetFilterGodotVersion" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="any">{{ t('assetLibrary.godotVersionAny') }}</option>
        <option value="4.0">{{ t('assetLibrary.godot4x') }}</option>
        <option value="3.0">{{ t('assetLibrary.godot3x') }}</option>
      </select>
      <select v-model="assetFilterSupport" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="">{{ t('assetLibrary.supportAll') }}</option>
        <option value="official">{{ t('assetLibrary.supportOfficial') }}</option>
        <option value="featured">{{ t('assetLibrary.supportFeatured') }}</option>
        <option value="community">{{ t('assetLibrary.supportCommunity') }}</option>
        <option value="testing">{{ t('assetLibrary.supportTesting') }}</option>
      </select>
      <select v-model="assetSortBy" @change="searchAssets()" class="px-2 py-1.5 border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary text-xs">
        <option value="updated">{{ t('assetLibrary.sortUpdated') }}</option>
        <option value="rating">{{ t('assetLibrary.sortRating') }}</option>
        <option value="name">{{ t('assetLibrary.sortName') }}</option>
        <option value="cost">{{ t('assetLibrary.sortCost') }}</option>
      </select>
    </div>

    <div v-if="selectedAssetIds.size > 0" class="bg-primary-50 dark:bg-primary-900/20 border border-primary-200 dark:border-primary-800 rounded-lg p-2 mb-3 flex items-center justify-between">
      <span class="text-xs font-medium text-primary-700 dark:text-primary-300">{{ t('assetLibrary.selectedCount', { count: selectedAssetIds.size }) }}</span>
      <button
        @click="batchImportAssets"
        :disabled="!!pluginStore.isImporting"
        class="px-3 py-1 bg-primary-600 text-white text-xs rounded-lg hover:bg-primary-700 disabled:opacity-50"
      >
        {{ t('assetLibrary.batchImport') }} ({{ selectedAssetIds.size }})
      </button>
    </div>

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
          'bg-white dark:bg-surface-card rounded-xl shadow hover:shadow-md p-3 transition-colors',
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
            <div class="flex items-center gap-2">
              <span class="font-medium text-gray-900 dark:text-content-primary text-sm truncate">{{ asset.title }}</span>
              <span v-if="asset.support_level === 'official'" class="px-1.5 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400">{{ t('assetLibrary.supportOfficial') }}</span>
              <span v-else-if="asset.support_level === 'featured'" class="px-1.5 py-0.5 rounded text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400">{{ t('assetLibrary.supportFeatured') }}</span>
            </div>
            <div class="text-xs text-gray-500 dark:text-content-secondary mt-0.5">
              {{ asset.author }} · {{ asset.category }} · {{ asset.cost }}
            </div>
          </div>
          <button
            v-if="!importedAssetIds.has(asset.asset_id)"
            @click="importAsset(asset.asset_id, asset.title)"
            :disabled="pluginStore.isImporting === asset.asset_id"
            class="btn-primary disabled:opacity-50 text-xs px-3 py-1.5 flex-shrink-0"
          >
            {{ pluginStore.isImporting === asset.asset_id ? t('assetLibrary.importing') : t('assetLibrary.import') }}
          </button>
          <span v-else class="text-xs px-3 py-1.5 text-green-600 dark:text-green-400 flex-shrink-0 font-medium">✓ {{ t('assetLibrary.alreadyImported') }}</span>
        </div>
      </div>
    </div>

    <div v-if="assetTotalPages > 0" class="flex items-center justify-between mt-4 pt-3 border-t border-gray-200 dark:border-surface-border">
      <span class="text-xs text-gray-500 dark:text-content-secondary">
        {{ t('assetLibrary.totalItems', { count: assetTotalItems }) }}
      </span>
      <div class="flex items-center gap-2">
        <button
          @click="assetPrevPage"
          :disabled="assetCurrentPage === 0"
          class="px-3 py-1 text-xs border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-card disabled:opacity-50"
        >
          {{ t('assetLibrary.prevPage') }}
        </button>
        <span class="text-xs text-gray-600 dark:text-content-secondary">
          {{ t('assetLibrary.page', { current: assetCurrentPage + 1, total: assetTotalPages }) }}
        </span>
        <button
          @click="assetNextPage"
          :disabled="assetCurrentPage >= assetTotalPages - 1"
          class="px-3 py-1 text-xs border border-gray-300 dark:border-surface-border rounded-lg bg-white dark:bg-surface-layer text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-card disabled:opacity-50"
        >
          {{ t('assetLibrary.nextPage') }}
        </button>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showAssetDetailDialog && assetDetail" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50" @click="showAssetDetailDialog = false; assetDetail = null">
        <div class="bg-white dark:bg-surface-card rounded-xl p-6 w-full max-w-lg shadow-xl max-h-[80vh] flex flex-col" @click.stop>
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-content-primary">{{ assetDetail.title }}</h3>
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
            <p class="text-sm text-gray-600 dark:text-content-secondary whitespace-pre-wrap bg-gray-50 dark:bg-surface-layer rounded-lg p-3">
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
              class="px-4 py-2 border border-gray-300 dark:border-surface-border rounded-lg text-gray-700 dark:text-content-primary text-sm hover:bg-gray-50 dark:hover:bg-surface-card"
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
  </div>
</template>
