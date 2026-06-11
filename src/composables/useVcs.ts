import { ref } from 'vue'
import { api } from '@/api'
import type { VcsInfo, VcsCommit, VcsDiffSummary } from '@/types'

export function useVcs() {
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const getVcsInfo = async (projectId: string): Promise<VcsInfo | null> => {
    try {
      return await api.getProjectVcsInfo(projectId)
    } catch (e) {
      console.warn('获取 VCS 信息失败:', e)
      return null
    }
  }

  const getVcsHistory = async (projectId: string, limit?: number): Promise<VcsCommit[]> => {
    try {
      return await api.getProjectVcsHistory(projectId, limit)
    } catch (e) {
      console.warn('获取提交历史失败:', e)
      return []
    }
  }

  const pull = async (projectId: string): Promise<string> => {
    isLoading.value = true
    error.value = null
    try {
      const result = await api.vcsPull(projectId)
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  const push = async (projectId: string): Promise<string> => {
    isLoading.value = true
    error.value = null
    try {
      const result = await api.vcsPush(projectId)
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  const commit = async (projectId: string, message: string, addAll?: boolean): Promise<string> => {
    isLoading.value = true
    error.value = null
    try {
      const result = await api.vcsCommit(projectId, message, addAll)
      return result
    } catch (e) {
      error.value = String(e)
      throw e
    } finally {
      isLoading.value = false
    }
  }

  const getDiff = async (projectId: string): Promise<VcsDiffSummary | null> => {
    try {
      return await api.vcsGetDiff(projectId)
    } catch (e) {
      console.warn('获取差异摘要失败:', e)
      return null
    }
  }

  const updateGitignore = async (projectId: string): Promise<void> => {
    try {
      await api.vcsUpdateGitignore(projectId)
    } catch (e) {
      console.warn('更新 .gitignore 失败:', e)
      throw e
    }
  }

  const batchGetVcsInfo = async (projectIds: string[]): Promise<Map<string, VcsInfo>> => {
    if (projectIds.length === 0) return new Map()
    try {
      const results = await api.batchGetVcsInfo(projectIds)
      const map = new Map<string, VcsInfo>()
      for (const [id, info] of results) {
        map.set(id, info)
      }
      return map
    } catch (e) {
      console.warn('批量获取 VCS 信息失败:', e)
      return new Map()
    }
  }

  return {
    isLoading,
    error,
    getVcsInfo,
    getVcsHistory,
    pull,
    push,
    commit,
    getDiff,
    updateGitignore,
    batchGetVcsInfo,
  }
}
