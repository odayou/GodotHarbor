import { ref, computed, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { useToast } from '@/composables/useToast'
import { usePluginStore } from '@/stores'
import { isOnline } from '@/composables/useNetworkStatus'
import type { StorePlugin, StoreSearchResult, StoreRecommendation, StoreCategory, OneClickInstallResult } from '@/types'

export function usePluginStoreComposable() {
  const toast = useToast()
  const { t } = useI18n()
  const pluginStore = usePluginStore()

  const searchQuery = ref('')
  const searchResults = ref<StorePlugin[]>([])
  const isSearching = ref(false)
  const categories = ref<StoreCategory[]>([])
  const selectedCategory = ref('')
  const sortBy = ref('updated')
  const godotVersionFilter = ref('')
  const currentPage = ref(1)
  const pageSize = ref(20)
  const totalResults = ref(0)
  const hasMore = ref(false)
  const hasSearched = ref(false)

  const recommendations = ref<StoreRecommendation[]>([])
  const isLoadingRecommendations = ref(false)

  const installProgress = ref<{
    assetId: number
    stage: 'downloading' | 'importing' | 'binding' | 'applying' | 'complete' | 'error'
    message: string
  } | null>(null)

  let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null

  onUnmounted(() => {
    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
      searchDebounceTimer = null
    }
  })

  const doSearch = async (immediate = false) => {
    if (!isOnline.value) {
      toast.warning(t('common.offlineNotice'))
      return
    }

    if (searchDebounceTimer) {
      clearTimeout(searchDebounceTimer)
    }

    const execute = async () => {
      isSearching.value = true
      try {
        const result: StoreSearchResult = await api.searchPluginStore({
          query: searchQuery.value,
          category: selectedCategory.value || undefined,
          godot_version: godotVersionFilter.value || undefined,
          sort_by: sortBy.value,
          page: currentPage.value,
          page_size: pageSize.value,
        })
        searchResults.value = result.plugins
        totalResults.value = result.total
        hasMore.value = result.has_more
        hasSearched.value = true
      } catch (error) {
        toast.error(t('common.loadFailed', { error }))
      } finally {
        isSearching.value = false
      }
    }

    if (immediate) {
      await execute()
    } else {
      searchDebounceTimer = setTimeout(execute, 400)
    }
  }

  const loadCategories = async () => {
    try {
      categories.value = await api.getPluginStoreCategoriesWithCounts()
    } catch (error) {
      console.error('Failed to load store categories:', error)
    }
  }

  const loadRecommendations = async (projectId?: string) => {
    if (!isOnline.value) return
    isLoadingRecommendations.value = true
    try {
      recommendations.value = await api.getPluginStoreRecommendations(projectId)
    } catch (error) {
      console.error('Failed to load recommendations:', error)
    } finally {
      isLoadingRecommendations.value = false
    }
  }

  const prevPage = () => {
    if (currentPage.value > 1) {
      currentPage.value--
      doSearch(true)
    }
  }

  const nextPage = () => {
    if (hasMore.value) {
      currentPage.value++
      doSearch(true)
    }
  }

  const totalPages = computed(() => Math.ceil(totalResults.value / pageSize.value))

  const oneClickInstall = async (
    assetId: number,
    assetName: string,
    projectId: string,
    autoApply: boolean = false
  ): Promise<OneClickInstallResult | null> => {
    installProgress.value = {
      assetId,
      stage: 'downloading',
      message: `正在下载 ${assetName}...`,
    }

    try {
      installProgress.value = {
        assetId,
        stage: 'importing',
        message: `正在导入 ${assetName}...`,
      }

      const result = await api.oneClickInstallPlugin(assetId, projectId, autoApply)

      if (result.success) {
        installProgress.value = {
          assetId,
          stage: 'complete',
          message: `${assetName} 安装完成`,
        }

        if (result.binding_created) {
          toast.success(t('plugins.importPluginSuccess', { name: assetName }))
        } else {
          toast.success(t('plugins.importPluginSuccess', { name: assetName }))
        }

        await pluginStore.loadPlugins()
        await doSearch(true)
      } else {
        installProgress.value = {
          assetId,
          stage: 'error',
          message: result.errors.join('; ') || '安装失败',
        }
        toast.error(result.errors.join('; ') || '安装失败')
      }

      return result
    } catch (error) {
      installProgress.value = {
        assetId,
        stage: 'error',
        message: String(error),
      }
      toast.error(t('assetLibrary.importFailed') + ': ' + error)
      return null
    } finally {
      setTimeout(() => {
        if (installProgress.value?.assetId === assetId) {
          installProgress.value = null
        }
      }, 2000)
    }
  }

  const simpleImport = async (assetId: number, assetName: string) => {
    if (!isOnline.value) {
      toast.warning(t('common.offlineNotice'))
      return
    }

    pluginStore.setImporting(String(assetId))
    try {
      const result = await api.importFromAssetLibraryWithProgress(String(assetId))
      toast.success(t('plugins.importPluginSuccess', { name: assetName }))
      await pluginStore.loadPlugins()
      await doSearch(true)
      return result
    } catch (error) {
      toast.error(t('assetLibrary.importFailed') + ': ' + error)
      return null
    } finally {
      pluginStore.resetImportProgress()
    }
  }

  const initStore = async (projectId?: string) => {
    await Promise.all([
      loadCategories(),
      loadRecommendations(projectId),
      doSearch(true),
    ])
  }

  return {
    searchQuery,
    searchResults,
    isSearching,
    categories,
    selectedCategory,
    sortBy,
    godotVersionFilter,
    currentPage,
    pageSize,
    totalResults,
    hasMore,
    hasSearched,
    recommendations,
    isLoadingRecommendations,
    installProgress,
    totalPages,
    doSearch,
    loadCategories,
    loadRecommendations,
    prevPage,
    nextPage,
    oneClickInstall,
    simpleImport,
    initStore,
  }
}
