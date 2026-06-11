import { ref } from 'vue'
import { api } from '@/api'
import { useToast } from '@/composables/useToast'
import type { EnvironmentSnapshot, GlobalUpgradeResult } from '@/types'

export function useBatchOps() {
  const toast = useToast()

  // ─── Snapshot Management ───
  const snapshots = ref<EnvironmentSnapshot[]>([])
  const isLoadingSnapshots = ref(false)
  const isCreatingSnapshot = ref(false)
  const isRestoringSnapshot = ref(false)

  const loadSnapshots = async (projectId: string) => {
    isLoadingSnapshots.value = true
    try {
      snapshots.value = await api.listSnapshots(projectId)
    } catch (error) {
      toast.error(`加载快照失败: ${error}`)
      snapshots.value = []
    } finally {
      isLoadingSnapshots.value = false
    }
  }

  const createSnapshot = async (projectId: string) => {
    isCreatingSnapshot.value = true
    try {
      const snapshot = await api.createSnapshot(projectId)
      toast.success(`快照已创建: ${snapshot.plugins.length} 个插件`)
      await loadSnapshots(projectId)
      return snapshot
    } catch (error) {
      toast.error(`创建快照失败: ${error}`)
      return null
    } finally {
      isCreatingSnapshot.value = false
    }
  }

  const restoreSnapshot = async (projectId: string, snapshotId: string) => {
    isRestoringSnapshot.value = true
    try {
      const restored = await api.restoreSnapshot(projectId, snapshotId)
      toast.success(`已从快照恢复: ${restored.length} 个插件`)
      return restored
    } catch (error) {
      toast.error(`恢复快照失败: ${error}`)
      return null
    } finally {
      isRestoringSnapshot.value = false
    }
  }

  const deleteSnapshot = async (snapshotId: string, projectId: string) => {
    try {
      await api.deleteSnapshot(snapshotId)
      toast.success('快照已删除')
      await loadSnapshots(projectId)
    } catch (error) {
      toast.error(`删除快照失败: ${error}`)
    }
  }

  // ─── Global Upgrade ───
  const upgradeResults = ref<GlobalUpgradeResult[]>([])
  const isUpgrading = ref(false)

  const globalUpgradePlugin = async (pluginId: string) => {
    isUpgrading.value = true
    try {
      upgradeResults.value = await api.globalUpgradePlugin(pluginId)
      const successCount = upgradeResults.value.filter(r => r.success).length
      const failCount = upgradeResults.value.filter(r => !r.success).length
      if (failCount > 0) {
        toast.warning(`全局升级完成: ${successCount} 成功, ${failCount} 失败`)
      } else {
        toast.success(`全局升级完成: ${successCount} 个项目已更新`)
      }
      return upgradeResults.value
    } catch (error) {
      toast.error(`全局升级失败: ${error}`)
      return null
    } finally {
      isUpgrading.value = false
    }
  }

  return {
    // Snapshot
    snapshots,
    isLoadingSnapshots,
    isCreatingSnapshot,
    isRestoringSnapshot,
    loadSnapshots,
    createSnapshot,
    restoreSnapshot,
    deleteSnapshot,
    // Global Upgrade
    upgradeResults,
    isUpgrading,
    globalUpgradePlugin,
  }
}
