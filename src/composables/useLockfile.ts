import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { api } from '@/api'
import { useToast } from '@/composables/useToast'
import type { HarborLock, LockVerifyResult, LockDiff } from '@/types'

export type LockStatus = 'locked_verified' | 'locked_drifted' | 'not_locked' | 'loading'

export function useLockfile(projectId: string) {
  const toast = useToast()
  const { t } = useI18n()

  const lock = ref<HarborLock | null>(null)
  const verifyResult = ref<LockVerifyResult | null>(null)
  const diff = ref<LockDiff | null>(null)
  const isLoading = ref(false)
  const isWriting = ref(false)
  const isSyncing = ref(false)
  const isVerifying = ref(false)
  const isRestoring = ref(false)

  const status = computed<LockStatus>(() => {
    if (isLoading.value) return 'loading'
    if (!lock.value) return 'not_locked'
    if (verifyResult.value) {
      return verifyResult.value.is_valid ? 'locked_verified' : 'locked_drifted'
    }
    return 'locked_verified'
  })

  const pluginCount = computed(() => lock.value?.plugins.length ?? 0)

  const lockedAt = computed(() => {
    if (!lock.value) return ''
    try {
      return new Date(lock.value.locked_at).toLocaleString()
    } catch {
      return lock.value.locked_at
    }
  })

  const hasDrift = computed(() => {
    if (!diff.value) return false
    return diff.value.added.length > 0 || diff.value.removed.length > 0 || diff.value.changed.length > 0
  })

  const readLock = async () => {
    isLoading.value = true
    try {
      lock.value = await api.readProjectLock(projectId)
    } catch {
      lock.value = null
    } finally {
      isLoading.value = false
    }
  }

  const generateAndWriteLock = async () => {
    isWriting.value = true
    try {
      await api.writeProjectLock(projectId)
      lock.value = await api.readProjectLock(projectId)
      verifyResult.value = null
      diff.value = null
      toast.success('已生成 harbor.lock')
    } catch (e: any) {
      toast.error(`生成锁文件失败: ${e?.toString() || e}`)
    } finally {
      isWriting.value = false
    }
  }

  const verify = async () => {
    isVerifying.value = true
    try {
      verifyResult.value = await api.verifyProjectLock(projectId)
      if (verifyResult.value.is_valid) {
        toast.success('锁文件验证通过')
      } else {
        toast.warning(`${verifyResult.value.mismatches.length} 个插件不匹配`)
      }
    } catch (e: any) {
      toast.error(`验证失败: ${e?.toString() || e}`)
      verifyResult.value = null
    } finally {
      isVerifying.value = false
    }
  }

  const computeDiff = async () => {
    try {
      diff.value = await api.diffProjectLock(projectId)
    } catch {
      diff.value = null
    }
  }

  const syncLock = async (strict = false) => {
    isSyncing.value = true
    try {
      const messages = await api.syncFromLock(projectId, strict)
      if (messages.length > 0) {
        toast.success(`同步完成: ${messages.length} 项操作`)
      } else {
        toast.info('无需同步，环境已一致')
      }
      await readLock()
      verifyResult.value = null
      diff.value = null
    } catch (e: any) {
      toast.error(`同步失败: ${e?.toString() || e}`)
    } finally {
      isSyncing.value = false
    }
  }

  const restore = async () => {
    isRestoring.value = true
    try {
      const r = await api.restoreProjectEnvironment(projectId)
      if (r.failed.length === 0 && r.missing.length === 0) {
        toast.success(t('lockfile.restoreSuccess', { ready: r.ready.length, imported: r.imported.length }))
      } else {
        toast.warning(t('lockfile.restorePartial', { ready: r.ready.length, imported: r.imported.length, failed: r.failed.length, missing: r.missing.length }))
      }
      await readLock()
      verifyResult.value = null
      diff.value = null
    } catch (e: any) {
      toast.error(t('lockfile.restoreFailed', { error: e?.toString() || e }))
    } finally {
      isRestoring.value = false
    }
  }

  return {
    lock,
    verifyResult,
    diff,
    status,
    pluginCount,
    lockedAt,
    hasDrift,
    isLoading,
    isWriting,
    isSyncing,
    isVerifying,
    isRestoring,
    readLock,
    generateAndWriteLock,
    verify,
    computeDiff,
    syncLock,
    restore,
  }
}
