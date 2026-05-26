import { ref, computed, onUnmounted, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { useToast } from '@/composables/useToast'
import { usePluginStore } from '@/stores'
import { sendAppNotification } from '@/composables/useNotification'
import { isOnline } from '@/composables/useNetworkStatus'
import type { Plugin, AssetLibrarySearchResult, AssetLibrarySearchResponse, AssetLibraryCategory, AssetLibraryAsset } from '@/types'

export function useAssetLibrary(options: {
  activeTab: Ref<'repository' | 'bindings' | 'assetLibrary'>
  loadPlugins: (force?: boolean) => Promise<void>
  showPostImportGuide: (pluginName: string, plugin?: Plugin) => Promise<void>
}) {
  const toast = useToast()
  const { t } = useI18n()
  const pluginStore = usePluginStore()

  const assetSearchQuery = ref('')
  const assetSearchResults = ref<AssetLibrarySearchResult[]>([])
  const isSearchingAssets = ref(false)
  const assetCategories = ref<AssetLibraryCategory[]>([])
  const assetFilterType = ref<string>('any')
  const assetFilterCategory = ref<string>('')
  const assetFilterGodotVersion = ref<string>('any')
  const assetFilterSupport = ref<string>('')
  const assetSortBy = ref<string>('updated')
  const assetCurrentPage = ref(0)
  const assetTotalPages = ref(0)
  const assetTotalItems = ref(0)
  const selectedAssetIds = ref<Set<string>>(new Set())
  let searchAbortController: AbortController | null = null
  const assetDetail = ref<AssetLibraryAsset | null>(null)
  const showAssetDetailDialog = ref(false)
  const searchCache = ref<Map<string, { data: AssetLibrarySearchResponse; timestamp: number }>>(new Map())
  const categoriesLoaded = ref(false)
  const hasSearched = ref(false)
  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

  onUnmounted(() => {
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
      searchDebounceTimer = null
    }
    searchCache.value.clear()
  })

  const plugins = computed(() => pluginStore.plugins)

  const importedAssetIds = computed(() => {
    const ids = new Set<string>()
    for (const p of plugins.value) {
      if (p.source.source_type === 'AssetLibrary' && p.source.url) {
        const match = p.source.url.match(/^asset-library:\/\/(\d+)$/)
        if (match) ids.add(match[1])
      }
    }
    return ids
  })

  const getCacheKey = () => {
    return JSON.stringify({
      filter: assetSearchQuery.value,
      type: assetFilterType.value,
      category: assetFilterCategory.value,
      support: assetFilterSupport.value,
      godot_version: assetFilterGodotVersion.value,
      sort: assetSortBy.value,
      page: assetCurrentPage.value
    })
  }

  const doSearch = async () => {
    if (!isOnline.value) {
      toast.warning(t('common.offlineNotice'))
      return
    }

    if (searchAbortController) {
      searchAbortController.abort()
    }
    searchAbortController = new AbortController()
    const currentController = searchAbortController

    isSearchingAssets.value = true
    try {
      const cacheKey = getCacheKey()
      const cached = searchCache.value.get(cacheKey)
      if (cached && Date.now() - cached.timestamp < 5 * 60 * 1000) {
        if (currentController.signal.aborted) return
        assetSearchResults.value = cached.data.result
        assetTotalPages.value = cached.data.pages
        assetTotalItems.value = cached.data.total_items
        hasSearched.value = true
        isSearchingAssets.value = false
        return
      }

      const result = await api.searchAssets({
        filter: assetSearchQuery.value || undefined,
        type: assetFilterType.value as 'any' | 'addon' | 'project',
        category: assetFilterCategory.value || undefined,
        support: assetFilterSupport.value || undefined,
        godot_version: assetFilterGodotVersion.value !== 'any' ? assetFilterGodotVersion.value : undefined,
        sort: assetSortBy.value as 'rating' | 'cost' | 'name' | 'updated',
        max_results: 20,
        page: assetCurrentPage.value
      })
      assetSearchResults.value = result.result
      assetTotalPages.value = result.pages
      assetTotalItems.value = result.total_items
      hasSearched.value = true
      if (searchCache.value.size > 50) {
        const oldest = Array.from(searchCache.value.entries()).sort((a, b) => a[1].timestamp - b[1].timestamp)
        for (let i = 0; i < oldest.length - 30; i++) {
          searchCache.value.delete(oldest[i][0])
        }
      }
      searchCache.value.set(cacheKey, { data: result, timestamp: Date.now() })
    } catch (error) {
      toast.error(t('common.loadFailed', { error }))
    } finally {
      isSearchingAssets.value = false
    }
  }

  const searchAssets = (immediate = false): Promise<void> => {
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }
    if (immediate) {
      return doSearch()
    } else {
      return new Promise(resolve => {
        searchDebounceTimer = setTimeout(() => {
          doSearch().then(resolve)
        }, 400)
      })
    }
  }

  const assetPrevPage = () => {
    if (assetCurrentPage.value > 0) {
      assetCurrentPage.value--
      searchAssets(true)
    }
  }

  const assetNextPage = () => {
    if (assetCurrentPage.value < assetTotalPages.value - 1) {
      assetCurrentPage.value++
      searchAssets(true)
    }
  }

  const toggleAssetSelection = (assetId: string) => {
    const newSet = new Set(selectedAssetIds.value)
    if (newSet.has(assetId)) {
      newSet.delete(assetId)
    } else {
      newSet.add(assetId)
    }
    selectedAssetIds.value = newSet
  }

  const importAsset = async (assetId: string, assetTitle: string) => {
    if (!isOnline.value) {
      toast.warning(t('common.offlineNotice'))
      return
    }

    pluginStore.setImporting(assetId)
    try {
      const result = await api.importFromAssetLibraryWithProgress(assetId)
      toast.success(t('plugins.importPluginSuccess', { name: assetTitle }))
      sendAppNotification('Godot Harbor', t('plugins.importPluginSuccess', { name: assetTitle }))
      await options.loadPlugins(true)
      options.showPostImportGuide(assetTitle, result)
    } catch (error) {
      toast.error(t('assetLibrary.importFailed') + ': ' + error)
    } finally {
      pluginStore.resetImportProgress()
    }
  }

  const batchImportAssets = async () => {
    const ids = Array.from(selectedAssetIds.value)
    if (ids.length === 0) return

    ids.forEach(id => pluginStore.setImporting(id))

    const results = await Promise.allSettled(
      ids.map(id => api.importFromAssetLibraryWithProgress(id))
    )

    pluginStore.resetImportProgress()
    selectedAssetIds.value = new Set()

    let successCount = 0
    let failCount = 0
    results.forEach(r => { if (r.status === 'fulfilled') successCount++; else failCount++ })

    if (failCount > 0) {
      toast.warning(t('plugins.depDialog.partialSuccess', { success: successCount, failed: failCount }))
      sendAppNotification('Godot Harbor', t('plugins.depDialog.partialSuccess', { success: successCount, failed: failCount }))
    } else {
      toast.success(t('common.batchImportSuccess', { count: successCount }))
      sendAppNotification('Godot Harbor', t('common.batchImportSuccess', { count: successCount }))
    }
    await options.loadPlugins(true)
  }

  const openAssetDetail = async (assetId: string) => {
    try {
      assetDetail.value = await api.getAssetDetailV2(assetId)
      showAssetDetailDialog.value = true
    } catch (error) {
      toast.error(t('common.loadFailed', { error }))
    }
  }

  const openPreviewLink = (url: string) => {
    window.open(url, '_blank')
  }

  const openAssetLibraryTab = async () => {
    const loadCategories = async () => {
      if (!categoriesLoaded.value || assetCategories.value.length === 0) {
        try {
          const config = await api.getAssetLibraryConfigure()
          assetCategories.value = config.categories || []
          categoriesLoaded.value = true
        } catch (error) {
          console.error('Failed to load categories:', error)
        }
      }
    }

    if (assetSearchResults.value.length === 0 && !isSearchingAssets.value && !hasSearched.value) {
      assetFilterSupport.value = 'featured'
      await Promise.all([loadCategories(), searchAssets(true)])
    } else {
      await loadCategories()
    }
  }

  const openAssetLibrary = async () => {
    options.activeTab.value = 'assetLibrary'
    assetSearchQuery.value = ''
    assetSearchResults.value = []
    selectedAssetIds.value = new Set()
    assetCurrentPage.value = 0
    assetTotalPages.value = 0
    assetTotalItems.value = 0
    hasSearched.value = false
    await openAssetLibraryTab()
  }

  return {
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
    openAssetLibrary,
    searchAssets,
    assetPrevPage,
    assetNextPage,
    toggleAssetSelection,
    importAsset,
    batchImportAssets,
    openAssetDetail,
    openPreviewLink,
    openAssetLibraryTab,
  }
}
