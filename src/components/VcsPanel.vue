<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useVcs } from '@/composables/useVcs'
import { useToast } from '@/composables/useToast'
import { api } from '@/api'
import type { VcsInfo, VcsCommit, VcsDiffSummary, VcsBranch } from '@/types'
import ConfirmDialog from '@/components/ConfirmDialog.vue'

const props = defineProps<{
  projectId: string
}>()

const { t } = useI18n()
const toast = useToast()
const { getVcsInfo, getVcsHistory, pull, push, commit, getDiff, updateGitignore } = useVcs()

const vcsInfo = ref<VcsInfo | null>(null)
const commits = ref<VcsCommit[]>([])
const diffSummary = ref<VcsDiffSummary | null>(null)
const commitMessage = ref('')
const stageAllChanges = ref(false)
const isExpanded = ref(true)
const isOperating = ref(false)
const branches = ref<VcsBranch[]>([])
const showBranchPanel = ref(false)
const newBranchName = ref('')
const switchingBranch = ref(false)
const showCheckoutConfirm = ref(false)
const pendingCheckoutBranch = ref('')

const loadVcsData = async () => {
  if (!props.projectId) return
  const info = await getVcsInfo(props.projectId)
  vcsInfo.value = info
  if (info && info.vcs_type === 'Git') {
    const [history, diff] = await Promise.all([
      getVcsHistory(props.projectId, 5),
      getDiff(props.projectId),
    ])
    commits.value = history
    diffSummary.value = diff
  }
}

const handlePull = async () => {
  isOperating.value = true
  try {
    const result = await pull(props.projectId)
    toast.success(result || t('vcs.pullSuccess'))
    await loadVcsData()
  } catch (e: any) {
    toast.error(`${t('vcs.pullFailed')}: ${e?.toString() || e}`)
  } finally {
    isOperating.value = false
  }
}

const handlePush = async () => {
  isOperating.value = true
  try {
    const result = await push(props.projectId)
    toast.success(result || t('vcs.pushSuccess'))
    await loadVcsData()
  } catch (e: any) {
    toast.error(`${t('vcs.pushFailed')}: ${e?.toString() || e}`)
  } finally {
    isOperating.value = false
  }
}

const handleCommit = async () => {
  if (!commitMessage.value.trim()) {
    toast.warning(t('vcs.commitMessageRequired'))
    return
  }
  isOperating.value = true
  try {
    const result = await commit(props.projectId, commitMessage.value.trim(), stageAllChanges.value)
    toast.success(result || t('vcs.commitSuccess'))
    commitMessage.value = ''
    await loadVcsData()
  } catch (e: any) {
    toast.error(`${t('vcs.commitFailed')}: ${e?.toString() || e}`)
  } finally {
    isOperating.value = false
  }
}

const handleUpdateGitignore = async () => {
  try {
    await updateGitignore(props.projectId)
    toast.success(t('vcs.gitignoreUpdated'))
  } catch (e: any) {
    toast.error(`${t('vcs.gitignoreUpdateFailed')}: ${e?.toString() || e}`)
  }
}

const loadBranches = async () => {
  try {
    branches.value = await api.vcsListBranches(props.projectId)
  } catch {
    branches.value = []
  }
}

const handleCheckout = (branch: string) => {
  pendingCheckoutBranch.value = branch
  showCheckoutConfirm.value = true
}

const confirmCheckout = async () => {
  const branch = pendingCheckoutBranch.value
  showCheckoutConfirm.value = false
  switchingBranch.value = true
  try {
    await api.vcsCheckout(props.projectId, branch)
    toast.success(t('vcs.checkoutSuccess', { branch }))
    await loadVcsData()
    branches.value = []
    showBranchPanel.value = false
  } catch (e: any) {
    toast.error(`${t('vcs.checkoutFailed')}: ${e?.toString() || e}`)
  } finally {
    switchingBranch.value = false
  }
}

const handleCreateBranch = async () => {
  if (!newBranchName.value.trim()) {
    toast.warning(t('vcs.branchNameRequired'))
    return
  }
  switchingBranch.value = true
  try {
    const name = newBranchName.value.trim()
    await api.vcsCreateBranch(props.projectId, name)
    toast.success(t('vcs.createBranchSuccess', { branch: name }))
    await loadBranches()
    newBranchName.value = ''
  } catch (e: any) {
    toast.error(`${t('vcs.createBranchFailed')}: ${e?.toString() || e}`)
  } finally {
    switchingBranch.value = false
  }
}

const getStatusLabel = (status: string): string => {
  const map: Record<string, string> = {
    Clean: t('vcs.statusClean'),
    Modified: t('vcs.statusModified'),
    Untracked: t('vcs.statusUntracked'),
    Ahead: t('vcs.statusAhead'),
    Behind: t('vcs.statusBehind'),
    Diverged: t('vcs.statusDiverged'),
    NoRemote: t('vcs.noRemote'),
  }
  return map[status] || status
}

const getStatusColor = (status: string): string => {
  switch (status) {
    case 'Clean': return 'text-green-600 dark:text-green-400'
    case 'Modified': return 'text-yellow-600 dark:text-yellow-400'
    case 'Diverged': return 'text-red-600 dark:text-red-400'
    case 'NoRemote': return 'text-gray-500 dark:text-gray-400'
    default: return 'text-blue-600 dark:text-blue-400'
  }
}

watch(() => props.projectId, () => {
  loadVcsData()
})

onMounted(() => {
  loadVcsData()
})
</script>

<template>
  <div v-if="vcsInfo && vcsInfo.vcs_type === 'Git'" class="border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] overflow-hidden">
    <button
      class="w-full flex items-center justify-between px-4 py-3 bg-gray-50 dark:bg-surface-hover hover:bg-gray-100 dark:hover:bg-surface-layer transition-colors"
      @click="isExpanded = !isExpanded"
    >
      <div class="flex items-center gap-2">
        <svg class="w-4 h-4 text-gray-600 dark:text-content-secondary" viewBox="0 0 16 16" fill="currentColor">
          <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
        </svg>
        <span class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ t('vcs.title') }}</span>
        <span class="text-xs font-mono px-1.5 py-0.5 rounded bg-gray-200 dark:bg-surface-layer text-gray-700 dark:text-content-secondary">{{ vcsInfo.branch }}</span>
        <span class="text-xs" :class="getStatusColor(vcsInfo.status)">{{ getStatusLabel(vcsInfo.status) }}</span>
      </div>
      <svg
        class="w-4 h-4 text-gray-500 transition-transform"
        :class="{ 'rotate-180': isExpanded }"
        fill="none" stroke="currentColor" viewBox="0 0 24 24"
      >
        <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <div v-if="isExpanded" class="p-3 space-y-3">
      <div class="grid grid-cols-2 gap-3">
        <div>
          <span class="text-xs text-gray-500 dark:text-content-muted">{{ t('vcs.branch') }}</span>
          <div class="flex items-center gap-1.5">
            <p class="text-sm font-medium text-gray-900 dark:text-content-primary font-mono">{{ vcsInfo.branch || '-' }}</p>
            <button
              @click="showBranchPanel = !showBranchPanel; if (showBranchPanel) loadBranches()"
              class="p-0.5 rounded hover:bg-gray-200 dark:hover:bg-surface-layer transition-colors"
              :title="t('vcs.switchBranch')"
            >
              <svg class="w-3.5 h-3.5 text-gray-400 dark:text-content-muted" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" /></svg>
            </button>
          </div>
        </div>
        <div>
          <span class="text-xs text-gray-500 dark:text-content-muted">{{ t('vcs.remote') }}</span>
          <p class="text-sm text-gray-900 dark:text-content-primary truncate" :title="vcsInfo.remote">{{ vcsInfo.remote || t('vcs.noRemote') }}</p>
        </div>
      </div>

      <div v-if="showBranchPanel" class="border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-3 space-y-2">
        <div class="flex gap-2">
          <input
            v-model="newBranchName"
            type="text"
            :placeholder="t('vcs.newBranchName')"
            class="flex-1 input-field"
            @keyup.enter="handleCreateBranch"
          />
          <button
            @click="handleCreateBranch"
            :disabled="switchingBranch || !newBranchName.trim()"
            class="px-2 py-1.5 text-xs font-medium rounded bg-green-600 text-white hover:bg-green-700 transition-colors disabled:opacity-50"
          >{{ t('vcs.createBranch') }}</button>
        </div>
        <div class="max-h-40 overflow-y-auto space-y-0.5">
          <button
            v-for="b in branches"
            :key="b.name + (b.is_remote ? '-r' : '-l')"
            @click="!b.is_current && !b.is_remote && handleCheckout(b.name)"
            :disabled="b.is_current || b.is_remote || switchingBranch"
            :class="[
              'w-full flex items-center justify-between px-2 py-1.5 rounded text-xs transition-colors',
              b.is_current
                ? 'bg-surface-hover text-brand-primary font-medium'
                : b.is_remote
                  ? 'text-gray-400 dark:text-content-muted cursor-not-allowed'
                  : 'text-gray-700 dark:text-content-secondary hover:bg-gray-100 dark:hover:bg-surface-layer'
            ]"
          >
            <span class="font-mono flex items-center gap-1.5">
              <span v-if="b.is_current" class="w-1.5 h-1.5 rounded-full bg-green-500"></span>
              <span v-if="b.is_remote" class="text-[10px] text-gray-400">remote/</span>
              {{ b.name }}
            </span>
            <span v-if="b.last_commit_date" class="text-[10px] text-gray-400 dark:text-content-muted">{{ b.last_commit_date }}</span>
          </button>
        </div>
      </div>

      <div class="flex gap-3 text-sm">
        <span v-if="vcsInfo.staged_files > 0" class="text-green-600 dark:text-green-400">
          <span class="font-medium">{{ vcsInfo.staged_files }}</span> {{ t('vcs.staged') }}
        </span>
        <span v-if="vcsInfo.modified_files > 0" class="text-yellow-600 dark:text-yellow-400">
          <span class="font-medium">{{ vcsInfo.modified_files }}</span> {{ t('vcs.statusModified') }}
        </span>
        <span v-if="vcsInfo.untracked_files > 0" class="text-blue-600 dark:text-blue-400">
          <span class="font-medium">{{ vcsInfo.untracked_files }}</span> {{ t('vcs.statusUntracked') }}
        </span>
        <span v-if="vcsInfo.ahead > 0" class="text-purple-600 dark:text-purple-400">
          ↑<span class="font-medium">{{ vcsInfo.ahead }}</span>
        </span>
        <span v-if="vcsInfo.behind > 0" class="text-orange-600 dark:text-orange-400">
          ↓<span class="font-medium">{{ vcsInfo.behind }}</span>
        </span>
        <span v-if="vcsInfo.staged_files === 0 && vcsInfo.modified_files === 0 && vcsInfo.untracked_files === 0" class="text-green-600 dark:text-green-400">
          {{ t('vcs.noChanges') }}
        </span>
      </div>

      <div class="flex gap-2">
        <button
          @click="handlePull"
          :disabled="isOperating"
          class="flex-1 px-3 py-2 text-sm font-medium rounded bg-blue-600 text-white hover:bg-blue-700 transition-colors disabled:opacity-50 flex items-center justify-center gap-1.5"
        >
          <svg v-if="isOperating" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" /></svg>
          <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
          {{ t('vcs.pull') }}
        </button>
        <button
          @click="handlePush"
          :disabled="isOperating"
          class="flex-1 px-3 py-2 text-sm font-medium rounded bg-purple-600 text-white hover:bg-purple-700 transition-colors disabled:opacity-50 flex items-center justify-center gap-1.5"
        >
          <svg v-if="isOperating" class="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" /><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" /></svg>
          <svg v-else class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 10l7-7m0 0l7 7m-7-7v18" /></svg>
          {{ t('vcs.push') }}
        </button>
      </div>

      <div class="space-y-2">
        <div class="flex gap-2">
          <input
            v-model="commitMessage"
            type="text"
            :placeholder="t('vcs.commitMessage')"
            class="flex-1 input-field"
            @keyup.enter="handleCommit"
          />
          <button
            @click="handleCommit"
            :disabled="isOperating || !commitMessage.trim()"
            class="px-4 py-2 text-sm font-medium rounded bg-green-600 text-white hover:bg-green-700 transition-colors disabled:opacity-50"
          >
            {{ t('vcs.commit') }}
          </button>
        </div>
        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            v-model="stageAllChanges"
            type="checkbox"
            class="w-3.5 h-3.5 rounded border-gray-300 dark:border-surface-border text-green-600 focus:ring-green-500"
          />
          <span class="text-xs text-gray-500 dark:text-content-muted">{{ t('vcs.stageAll') }}</span>
        </label>
      </div>

      <div v-if="diffSummary && diffSummary.files.length > 0" class="space-y-1">
        <span class="text-xs text-gray-500 dark:text-content-muted">{{ t('vcs.changedFiles') }} ({{ t('vcs.diffSummary', { added: diffSummary.added, modified: diffSummary.modified, deleted: diffSummary.deleted }) }})</span>
        <div class="max-h-32 overflow-y-auto space-y-0.5">
          <div
            v-for="file in diffSummary.files"
            :key="file.path"
            class="flex items-center gap-2 text-xs py-0.5"
          >
            <span
              class="w-4 text-center font-mono font-bold flex-shrink-0"
              :class="{
                'text-green-600 dark:text-green-400': file.status === 'added',
                'text-yellow-600 dark:text-yellow-400': file.status === 'modified',
                'text-red-600 dark:text-red-400': file.status === 'deleted',
                'text-blue-600 dark:text-blue-400': file.status === 'renamed',
              }"
            >
              {{ file.status === 'added' ? 'A' : file.status === 'modified' ? 'M' : file.status === 'deleted' ? 'D' : 'R' }}
            </span>
            <span class="text-gray-700 dark:text-content-secondary truncate font-mono">{{ file.path }}</span>
          </div>
        </div>
      </div>

      <div v-if="commits.length > 0" class="space-y-2">
        <span class="text-xs text-gray-500 dark:text-content-muted">{{ t('vcs.recentCommits') }}</span>
        <div class="space-y-1.5 max-h-40 overflow-y-auto">
          <div
            v-for="c in commits"
            :key="c.hash"
            class="flex items-start gap-2 text-xs"
          >
            <span class="font-mono text-primary-600 dark:text-brand-primary flex-shrink-0">{{ c.short_hash }}</span>
            <div class="min-w-0 flex-1">
              <p class="text-gray-900 dark:text-content-primary truncate">{{ c.message.split('\n')[0] }}</p>
              <p class="text-gray-400 dark:text-content-muted">{{ c.author }} · {{ c.date }}</p>
            </div>
          </div>
        </div>
      </div>

      <button
        @click="handleUpdateGitignore"
        class="w-full px-3 py-2 text-sm rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-secondary hover:bg-gray-50 dark:hover:bg-surface-hover transition-colors flex items-center justify-center gap-1.5"
      >
        <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>
        {{ t('vcs.updateGitignore') }}
      </button>
    </div>
  </div>
  <div v-else-if="vcsInfo" class="border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-3">
    <div class="flex items-center gap-2 text-gray-500 dark:text-content-muted">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
      <span class="text-sm">{{ t('vcs.notGitProject') }}</span>
    </div>
  </div>
  <div v-else class="border border-gray-200/60 dark:border-surface-border/40 rounded-[6px] p-4">
    <div class="flex items-center gap-2 text-gray-400 dark:text-content-muted">
      <div class="animate-spin rounded-full h-4 w-4 border-2 border-gray-300 border-t-transparent"></div>
      <span class="text-sm">{{ t('common.loading') }}</span>
    </div>
  </div>
  <ConfirmDialog
    v-model="showCheckoutConfirm"
    :title="t('vcs.switchBranch')"
    :description="t('vcs.checkoutConfirm', { branch: pendingCheckoutBranch })"
    :confirmText="t('vcs.switchBranch')"
    confirmColor="blue"
    @confirm="confirmCheckout"
  />
</template>
