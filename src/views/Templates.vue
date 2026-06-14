<script setup lang="ts">
import { ref, onMounted, computed, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { api } from '@/api'
import type { Template, TemplateCategory, TemplateInstantiationProgress, Project } from '@/types'
import { open } from '@tauri-apps/plugin-dialog'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useToast } from '@/composables/useToast'
import { useEngineLauncher } from '@/composables/useEngineLauncher'
import { useFileManager } from '@/composables/useFileManager'
import { useDialogEscape } from '@/composables/useDialogEscape'
import { isOnline } from '@/composables/useNetworkStatus'
import { useContextMenu } from '@/composables/useContextMenu'
import type { ContextMenuEntry } from '@/composables/useContextMenu'
import EmptyState from '@/components/EmptyState.vue'
import ContextMenu from '@/components/ContextMenu.vue'
import SkeletonList from '@/components/SkeletonList.vue'
import ConfirmDialog from '@/components/ConfirmDialog.vue'
import ProjectSelector from '@/components/ProjectSelector.vue'
import TemplateExportDialog from '@/components/TemplateExportDialog.vue'
import TemplateImportDialog from '@/components/TemplateImportDialog.vue'
import KeypairManager from '@/components/KeypairManager.vue'

const toast = useToast()
const { t } = useI18n()
const route = useRoute()
const {
  openProjectWithEngine,
  showEngineSelectDialog,
  engineSelectProject,
  matchedEngines,
  isLoadingEngines,
  isLaunching,
  launchWithEngine,
  closeEngineSelectDialog,
  getMatchLevelClass,
  getMatchLevelLabel,
  getMatchLevelDesc,
} = useEngineLauncher()
const { openInFileManager } = useFileManager()

const templates = ref<Template[]>([])
const showCreateHint = ref(false)
const isLoading = ref(true)
const isRefreshing = ref(false)
const loadError = ref<string | null>(null)
const categoryFilter = ref<TemplateCategory | 'all'>('all')
const searchQuery = ref('')

const showDetailDialog = ref(false)
const selectedTemplate = ref<Template | null>(null)

const showCreateDialog = ref(false)
const createProjectName = ref('')
const createTargetDir = ref('')
const enableMobileSupport = ref(false)
const isCreating = ref(false)
const createProgress = ref<TemplateInstantiationProgress | null>(null)
const projectNameError = ref('')

const isValidProjectName = computed(() => {
  const name = createProjectName.value.trim()
  if (!name) return false
  if (/[<>:"/\\|?*]/.test(name)) return false
  if (name.startsWith('.') || name.endsWith('.')) return false
  if (name.length > 200) return false
  return true
})

const validateProjectName = () => {
  const name = createProjectName.value.trim()
  if (!name) {
    projectNameError.value = ''
    return
  }
  if (/[<>:"/\\|?*]/.test(name)) {
    projectNameError.value = t('templates.invalidChars')
  } else if (name.startsWith('.') || name.endsWith('.')) {
    projectNameError.value = t('templates.invalidStartEnd')
  } else {
    projectNameError.value = ''
  }
}

const isValidImportUrl = computed(() => {
  const url = importUrl.value.trim()
  if (!url) return false
  try {
    const parsed = new URL(url)
    return parsed.protocol === 'http:' || parsed.protocol === 'https:'
  } catch {
    return false
  }
})

const showImportDialog = ref(false)
const importUrl = ref('')
const isImporting = ref(false)

const showGenerateFromProjectDialog = ref(false)
const generateProjectId = ref('')
const generateTemplateName = ref('')
const generateCategory = ref<TemplateCategory>('Custom')
const isGenerating = ref(false)
const projects = ref<Project[]>([])

const showDeleteConfirm = ref(false)
const deleteTargetId = ref('')

const showExportDialog = ref(false)
const exportTemplate = ref<Template | null>(null)
const showImportFileDialog = ref(false)
const showKeypairManager = ref(false)

useDialogEscape(showDetailDialog)
useDialogEscape(showCreateDialog)
useDialogEscape(showImportDialog)
useDialogEscape(showGenerateFromProjectDialog)

const templateContextMenu = useContextMenu()

const showTemplateContextMenu = (event: MouseEvent, tpl: Template) => {
  event.stopPropagation()
  templateContextMenu.show(event, [
    {
      label: t('templates.contextMenu.createProject'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 4v16m8-8H4" /></svg>',
      action: () => openCreateDialog(tpl),
    },
    { separator: true },
    {
      label: t('templates.contextMenu.exportTemplate'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" /></svg>',
      action: () => { exportTemplate.value = tpl; showExportDialog.value = true },
    },
    { separator: true },
    {
      label: t('templates.contextMenu.deleteTemplate'),
      icon: '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>',
      action: () => { deleteTargetId.value = tpl.template_id; showDeleteConfirm.value = true },
      disabled: tpl.is_builtin,
      danger: true,
    },
  ] as ContextMenuEntry[])
}

let unlistenProgress: UnlistenFn | null = null

onMounted(async () => {
  const [, projectList] = await Promise.all([
    loadTemplates(),
    api.getProjects().catch(() => [] as Project[])
  ])
  projects.value = projectList
  unlistenProgress = await listen('template-instantiation-progress', (event) => {
    createProgress.value = event.payload as TemplateInstantiationProgress
  })
  if (route.query.action === 'create' && templates.value.length > 0) {
    showCreateHint.value = true
  }
})

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress()
  }
})

const loadTemplates = async (force = false) => {
  const hasData = templates.value.length > 0
  if (hasData && !force) {
    isRefreshing.value = true
    loadError.value = null
    try {
      await api.ensureBuiltinTemplates()
      templates.value = await api.listHubTemplates()
    } catch (e: any) {
      loadError.value = e?.toString() || 'Failed to load templates'
    } finally {
      isRefreshing.value = false
    }
    return
  }
  isLoading.value = true
  loadError.value = null
  try {
    await api.ensureBuiltinTemplates()
    templates.value = await api.listHubTemplates()
  } catch (e: any) {
    loadError.value = e?.toString() || 'Failed to load templates'
  } finally {
    isLoading.value = false
  }
}

const filteredTemplates = computed(() => {
  return templates.value.filter(tpl => {
    const matchesCategory = categoryFilter.value === 'all' || tpl.category === categoryFilter.value
    const matchesSearch = searchQuery.value === '' ||
      tpl.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      tpl.description.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      tpl.tags.some(tag => tag.toLowerCase().includes(searchQuery.value.toLowerCase()))
    return matchesCategory && matchesSearch
  })
})

const categories = computed(() => {
  const cats = new Set<TemplateCategory>(templates.value.map(t => t.category))
  return ['all', ...Array.from(cats)] as const
})

const categoryIcon = (cat: TemplateCategory | 'all') => {
  switch (cat) {
    case 'all': return '📋'
    case 'Starter2D': return '🎮'
    case 'Starter3D': return '🌐'
    case 'RPG': return '⚔️'
    case 'Platformer': return '🏃'
    case 'Multiplayer': return '👥'
    case 'Mobile': return '📱'
    case 'Blank': return '📄'
    case 'Custom': return '🔧'
    default: return '📁'
  }
}

const openDetail = (tpl: Template) => {
  selectedTemplate.value = tpl
  showDetailDialog.value = true
}

const openCreateDialog = async (tpl: Template) => {
  selectedTemplate.value = tpl
  createProjectName.value = ''
  createProgress.value = null
  projectNameError.value = ''
  enableMobileSupport.value = false
  if (!createTargetDir.value) {
    try {
      const paths = await api.getStoragePaths()
      const docsDir = paths.app_data_dir.replace(/[/\\]GodotHarbor[/\\]?$/, '')
      createTargetDir.value = docsDir
    } catch {
      createTargetDir.value = ''
    }
  }
  showCreateDialog.value = true
}

const selectTargetDir = async () => {
  const selected = await open({ directory: true, multiple: false, title: t('projects.selectDir') || 'Select Directory' })
  if (selected) {
    createTargetDir.value = selected as string
  }
}

const lastCreatedProjectId = ref('')
const lastCreatedProject = ref<Project | null>(null)

const handleCreate = async () => {
  if (!selectedTemplate.value || !createProjectName.value.trim() || !createTargetDir.value.trim()) return
  if (!isValidProjectName.value) return

  isCreating.value = true
  createProgress.value = null
  try {
    const result = await api.instantiateTemplate(
      selectedTemplate.value.template_id,
      createProjectName.value.trim(),
      createTargetDir.value.trim(),
      enableMobileSupport.value
    )
    showCreateDialog.value = false
    lastCreatedProjectId.value = result.project_id

    // 获取创建的项目信息用于成功提示
    try {
      const allProjects = await api.getProjects()
      lastCreatedProject.value = allProjects.find(p => p.project_id === result.project_id) || null
    } catch {
      lastCreatedProject.value = null
    }

    if (result.failed_plugins.length > 0) {
      const details = result.failed_plugins.join('\n')
      toast.warning(`${t('templates.createSuccess')} (${result.failed_plugins.length} ${t('templates.partialFailed')}):\n${details}`, 8000)
    } else {
      toast.success(t('templates.createSuccess'))
    }

    if (result.engine_installed) {
      try {
        const projects = await api.getProjects()
        const project = projects.find(p => p.project_id === result.project_id)
        if (project) {
          await openProjectWithEngine(project)
        }
      } catch {
        // 自动打开失败不影响创建结果
      }
    }
  } catch (e: any) {
    toast.error(`${t('templates.createFailed')}: ${e?.toString() || e}`)
  } finally {
    isCreating.value = false
  }
}

const handleImport = async () => {
  if (!importUrl.value.trim()) return
  if (!isOnline.value) {
    toast.error(t('common.offlineError'))
    return
  }
  isImporting.value = true
  try {
    await api.importTemplateFromUrl(importUrl.value.trim())
    toast.success(t('templates.importSuccess'))
    showImportDialog.value = false
    importUrl.value = ''
    await loadTemplates()
  } catch (e: any) {
    toast.error(`${t('templates.importFailed')}: ${e?.toString() || e}`)
  } finally {
    isImporting.value = false
  }
}

const handleDelete = async () => {
  try {
    await api.deleteHubTemplate(deleteTargetId.value)
    toast.success(t('templates.deleteSuccess') || t('templates.saveSuccess'))
    await loadTemplates()
  } catch (e: any) {
    toast.error(t('templates.deleteFailed', { error: e?.toString() || e }))
  }
  showDeleteConfirm.value = false
  deleteTargetId.value = ''
}

const handleGenerateFromProject = async () => {
  if (!generateProjectId.value || !generateTemplateName.value.trim()) return
  const nameExists = templates.value.some(t => t.name === generateTemplateName.value.trim())
  if (nameExists) {
    toast.error(t('templates.nameExists'))
    return
  }
  isGenerating.value = true
  try {
    // 前端使用 PascalCase 分类名，后端期望 snake_case
    const categoryMap: Record<string, string> = {
      Starter2D: 'starter_2d',
      Starter3D: 'starter_3d',
      RPG: 'rpg',
      Platformer: 'platformer',
      Multiplayer: 'multiplayer',
      Mobile: 'mobile',
      Blank: 'blank',
      Custom: 'custom',
    }
    const backendCategory = categoryMap[generateCategory.value] || 'custom'
    await api.generateTemplateFromProject(generateProjectId.value, generateTemplateName.value.trim(), backendCategory)
    toast.success(t('templates.generateSuccess'))
    showGenerateFromProjectDialog.value = false
    generateProjectId.value = ''
    generateTemplateName.value = ''
    generateCategory.value = 'Custom'
    await loadTemplates()
  } catch (e: any) {
    toast.error(`${t('templates.generateFailed')}: ${e?.toString() || e}`)
  } finally {
    isGenerating.value = false
  }
}

const progressPercent = computed(() => {
  if (!createProgress.value) return 0
  return Math.round(createProgress.value.progress * 100)
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div v-if="showCreateHint" class="mx-3 mt-3 px-3 py-2 bg-primary-50 dark:bg-surface-hover border border-primary-200 dark:border-surface-border rounded flex items-center justify-between">
      <span class="text-sm text-primary-700 dark:text-content-secondary">{{ t('templates.selectToCreate') }}</span>
      <button @click="showCreateHint = false" class="text-primary-500 hover:text-primary-700 dark:hover:text-brand-primary">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="shrink-0 px-3 pb-3">
      <div class="flex items-center justify-between mb-3">
        <div>
          <h1 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ t('templates.title') }}</h1>
          <p class="text-xs text-gray-500 dark:text-content-muted mt-0.5">{{ t('templates.subtitle') }}</p>
        </div>
        <div class="flex items-center gap-2">
          <button
            @click="showImportFileDialog = true"
            class="px-3 py-1 text-xs font-medium rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors"
          >
            {{ t('templates.importFile') }}
          </button>
          <button
            @click="showImportDialog = true"
            class="px-3 py-1 text-xs font-medium rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors"
          >
            {{ t('templates.importUrl') }}
          </button>
          <button
            @click="showKeypairManager = true"
            class="px-3 py-1 text-xs font-medium rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors"
            :title="t('templates.keypairManage')"
          >
            <svg class="w-4 h-4 inline-block mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
            </svg>
            {{ t('templates.keypair') }}
          </button>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <div class="flex-1 relative">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="t('projects.search')"
            class="input-field pl-10"
          />
        </div>
        <div class="flex gap-1 flex-wrap">
          <button
            v-for="cat in categories"
            :key="cat"
            @click="categoryFilter = cat"
            :class="categoryFilter === cat ? 'filter-btn-active' : 'filter-btn'"
          >
            {{ categoryIcon(cat) }} {{ cat === 'all' ? t('templates.category.all') : t(`templates.category.${cat}`) }}
          </button>
        </div>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-3 pb-3">
      <SkeletonList v-if="isLoading" :count="4" />

      <div v-else-if="loadError" class="text-center py-12">
        <svg class="w-12 h-12 mx-auto text-red-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        <p class="text-red-500 text-sm mb-2">{{ loadError }}</p>
        <button
          class="px-4 py-2 text-sm font-medium text-primary-600 hover:text-primary-700 bg-primary-50 hover:bg-primary-100 dark:bg-surface-hover dark:text-brand-primary dark:hover:bg-surface-hover rounded transition-colors"
          @click="loadTemplates(true)"
        >
          {{ t('common.retry') }}
        </button>
      </div>

      <EmptyState
        v-else-if="filteredTemplates.length === 0"
        :title="t('templates.empty')"
        :description="t('templates.emptyDesc')"
        icon="template"
        :shortcuts="[
          { key: 'Ctrl+K', description: t('sidebar.openCommandPaletteShortcut') },
        ]"
      />

      <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        <div
          v-for="tpl in filteredTemplates"
          :key="tpl.template_id"
          class="group relative bg-white dark:bg-surface-card rounded border border-gray-200/80 dark:border-surface-border/60 hover:border-primary-300 dark:hover:border-primary-800/40 transition-all duration-200 cursor-pointer overflow-hidden"
          @click="openDetail(tpl)"
          @contextmenu="showTemplateContextMenu($event, tpl)"
          >
          <div class="p-3">
            <div class="flex items-start justify-between mb-2">
              <div class="flex items-center gap-2">
                <div class="w-7 h-7 rounded bg-primary-50 dark:bg-surface-hover flex items-center justify-center text-base">
                  {{ categoryIcon(tpl.category) }}
                </div>
                <div>
                  <h3 class="font-semibold text-gray-900 dark:text-content-primary text-sm">{{ tpl.name }}</h3>
                  <p class="text-xs text-gray-500 dark:text-content-muted">{{ t(`templates.category.${tpl.category}`) }}</p>
                </div>
              </div>
              <span
                v-if="tpl.is_builtin"
                class="px-1.5 py-0.5 text-[11px] font-medium rounded bg-primary-50 dark:bg-surface-hover text-primary-600 dark:text-brand-primary"
              >
                {{ t('templates.builtin') }}
              </span>
            </div>

            <p class="text-xs text-gray-600 dark:text-content-secondary line-clamp-2 mb-2">{{ tpl.description }}</p>

            <div class="flex items-center gap-2 text-xs text-gray-500 dark:text-content-muted mb-3">
              <span class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
                {{ tpl.godot.version }}
              </span>
              <span v-if="tpl.plugins.length > 0" class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                </svg>
                {{ tpl.plugins.length }} {{ t('templates.plugins') }}
              </span>
              <span class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
                {{ tpl.directories.length }} {{ t('templates.directories') }}
              </span>
            </div>

            <div v-if="tpl.plugins.length > 0" class="mb-2">
              <span class="text-[11px] text-gray-500 dark:text-content-muted">
                {{ tpl.plugins.slice(0, 3).map(p => p.name).join(', ') }}<span v-if="tpl.plugins.length > 3"> +{{ tpl.plugins.length - 3 }}</span>
              </span>
            </div>

            <div class="flex flex-wrap gap-1.5 mb-3">
              <span
                v-for="tag in tpl.tags.slice(0, 4)"
                :key="tag"
                class="px-1.5 py-0.5 text-[11px] rounded bg-gray-100 dark:bg-surface-layer text-gray-600 dark:text-content-secondary"
              >
                {{ tag }}
              </span>
            </div>

            <div class="flex gap-2">
              <button
                @click.stop="openCreateDialog(tpl)"
                class="flex-1 py-1.5 text-xs font-medium btn-primary transition-colors"
              >
                {{ t('templates.createProject') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail Dialog -->
    <Teleport to="body">
      <div v-if="showDetailDialog && selectedTemplate" class="fixed inset-0 z-[60] flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="showDetailDialog = false"></div>
        <div class="dialog-container relative z-10 max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto">
          <div class="p-4">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-2">
                <div class="w-10 h-10 rounded bg-primary-50 dark:bg-surface-hover flex items-center justify-center text-xl">
                  {{ categoryIcon(selectedTemplate.category) }}
                </div>
                <div>
                  <h2 class="text-sm font-semibold text-gray-900 dark:text-content-primary">{{ selectedTemplate.name }}</h2>
                  <p class="text-sm text-gray-500 dark:text-content-muted">
                    {{ t(`templates.category.${selectedTemplate.category}`) }}
                    <span v-if="selectedTemplate.author"> · {{ selectedTemplate.author }}</span>
                  </p>
                </div>
              </div>
              <button @click="showDetailDialog = false" class="p-1.5 rounded hover:bg-gray-100 dark:hover:bg-surface-layer text-gray-500">
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <p class="text-sm text-gray-600 dark:text-content-secondary mb-3">{{ selectedTemplate.description }}</p>

            <div v-if="selectedTemplate.preview_images && selectedTemplate.preview_images.length > 0" class="mb-3">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.previewImages') }}</h3>
              <div class="grid grid-cols-2 gap-2">
                <img
                  v-for="(img, idx) in selectedTemplate.preview_images"
                  :key="idx"
                  :src="img"
                  :alt="`Preview ${idx + 1}`"
                  class="w-full h-32 object-cover rounded border border-gray-200/60 dark:border-surface-border/40"
                  @error="($event.target as HTMLImageElement).style.display = 'none'"
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-3 mb-3">
              <div class="p-2.5 rounded-[6px] bg-gray-50 dark:bg-surface-layer">
                <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.godotVersion') }}</p>
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ selectedTemplate.godot.version }}{{ selectedTemplate.godot.mono ? ' (Mono)' : '' }}</p>
              </div>
              <div class="p-3 rounded-[6px] bg-gray-50 dark:bg-surface-layer">
                <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.plugins') }}</p>
                <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ selectedTemplate.plugins.length }}</p>
              </div>
            </div>

            <div v-if="selectedTemplate.plugins.length > 0" class="mb-3">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.plugins') }}</h3>
              <div class="space-y-2">
                <div
                  v-for="plugin in selectedTemplate.plugins"
                  :key="plugin.name"
                  class="flex items-center justify-between p-2 rounded-[4px] bg-gray-50 dark:bg-surface-layer"
                >
                  <div>
                    <p class="text-sm font-medium text-gray-900 dark:text-content-primary">{{ plugin.name }}</p>
                    <p class="text-xs text-gray-500 dark:text-content-muted">v{{ plugin.version }} · {{ plugin.source }}</p>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="selectedTemplate.directories.length > 0" class="mb-3">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.directories') }}</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="dir in selectedTemplate.directories"
                  :key="dir.path"
                  class="px-2 py-0.5 text-xs rounded-[4px] bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary font-mono"
                >
                  {{ dir.path }}
                </span>
              </div>
            </div>

            <div v-if="selectedTemplate.export_presets.length > 0" class="mb-3">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.exportPresets') }}</h3>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="preset in selectedTemplate.export_presets"
                  :key="preset.name"
                  class="px-2 py-0.5 text-xs rounded-[4px] bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary"
                >
                  {{ preset.name }} ({{ preset.platform }})
                </span>
              </div>
            </div>

            <div v-if="selectedTemplate.project_config" class="mb-3">
              <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-2">{{ t('templates.projectConfig') }}</h3>
              <div class="space-y-2">
                <div v-if="selectedTemplate.project_config.autoloads && Object.keys(selectedTemplate.project_config.autoloads).length > 0">
                  <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.autoloads') }}</p>
                  <div class="flex flex-wrap gap-1.5">
                    <span
                      v-for="(path, name) in selectedTemplate.project_config.autoloads"
                      :key="String(name)"
                      class="px-1.5 py-0.5 text-[11px] rounded bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary font-mono"
                    >
                      {{ name }}
                    </span>
                  </div>
                </div>
                <div v-if="selectedTemplate.project_config.layer_names">
                  <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.layerNames') }}</p>
                  <div class="space-y-1">
                    <div v-for="(layers, domain) in selectedTemplate.project_config.layer_names" :key="String(domain)" class="flex flex-wrap gap-1.5 items-center">
                      <span class="text-[11px] text-gray-400 dark:text-content-muted min-w-[60px]">{{ domain }}:</span>
                      <span
                        v-for="(layer, i) in (Array.isArray(layers) ? layers : [])"
                        :key="i"
                        class="px-1.5 py-0.5 text-[11px] rounded bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary"
                      >
                        {{ layer }}
                      </span>
                    </div>
                  </div>
                </div>
                <div v-if="selectedTemplate.project_config.input_mappings && Object.keys(selectedTemplate.project_config.input_mappings).length > 0">
                  <p class="text-xs text-gray-500 dark:text-content-muted mb-1">{{ t('templates.inputMappings') }}</p>
                  <div class="flex flex-wrap gap-1.5">
                    <span
                      v-for="(mapping, name) in selectedTemplate.project_config.input_mappings"
                      :key="String(name)"
                      class="px-1.5 py-0.5 text-[11px] rounded bg-gray-50 dark:bg-surface-layer text-gray-700 dark:text-content-secondary"
                    >
                      {{ name }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div class="flex gap-2">
              <button
                @click="showDetailDialog = false; openCreateDialog(selectedTemplate!)"
                class="flex-1 py-2 text-sm font-medium btn-primary transition-colors"
              >
                {{ t('templates.createProject') }}
              </button>
              <button
                @click="exportTemplate = selectedTemplate; showDetailDialog = false; showExportDialog = true"
                class="px-3 py-2 text-sm font-medium rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors"
              >
                {{ t('templates.export') }}
              </button>
              <button
                v-if="!selectedTemplate.is_builtin"
                @click="deleteTargetId = selectedTemplate.template_id; showDetailDialog = false; showDeleteConfirm = true"
                class="px-3 py-2 text-sm font-medium rounded border border-red-300 dark:border-red-800 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
              >
                {{ t('common.delete') || 'Delete' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Create Project Dialog -->
    <Teleport to="body">
      <div v-if="showCreateDialog && selectedTemplate" class="fixed inset-0 z-[70] flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isCreating && (showCreateDialog = false)"></div>
        <div class="dialog-container relative z-10 max-w-md w-full mx-4">
          <div class="p-4">
            <h2 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">
              {{ t('templates.createProject') }} — {{ selectedTemplate.name }}
            </h2>

            <div class="space-y-3">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.projectName') }}</label>
                <input
                  v-model="createProjectName"
                  type="text"
                  :disabled="isCreating"
                  @input="validateProjectName"
                  class="w-full px-3 py-2 text-sm rounded border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-1 focus:ring-primary-500 outline-none disabled:opacity-50"
                  :class="{ 'border-red-400 dark:border-red-500': projectNameError }"
                />
                <p v-if="projectNameError" class="mt-1 text-xs text-red-500">{{ projectNameError }}</p>
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('projects.targetDir') }}</label>
                <div class="flex gap-2">
                  <input
                    v-model="createTargetDir"
                    type="text"
                    :disabled="isCreating"
                    class="flex-1 px-3 py-2 text-sm rounded border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-1 focus:ring-primary-500 outline-none disabled:opacity-50"
                  />
                  <button
                    @click="selectTargetDir"
                    :disabled="isCreating"
                    class="px-3 py-2 text-sm rounded border border-gray-300 dark:border-surface-border hover:bg-gray-50 dark:hover:bg-surface-layer disabled:opacity-50"
                  >
                    ...
                  </button>
                </div>
              </div>
              <div class="flex items-center gap-2">
                <input
                  id="create-mobile-support"
                  v-model="enableMobileSupport"
                  type="checkbox"
                  :disabled="isCreating"
                  class="checkbox-field"
                />
                <label for="create-mobile-support" class="text-sm text-gray-700 dark:text-content-secondary cursor-pointer">
                  {{ t('templates.enableMobileSupport') }}
                </label>
              </div>
            </div>

            <div v-if="createProgress" class="mt-4">
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs text-gray-500 dark:text-content-muted">{{ createProgress.message }}</span>
                <span class="text-xs font-medium text-primary-600 dark:text-brand-primary">{{ progressPercent }}%</span>
              </div>
              <div class="w-full bg-gray-200 dark:bg-surface-border rounded-full h-1.5">
                <div
                  class="bg-primary-600 h-1.5 rounded-full transition-all duration-300"
                  :style="{ width: `${progressPercent}%` }"
                ></div>
              </div>
            </div>

            <div class="flex gap-2 mt-4">
              <button
                @click="showCreateDialog = false"
                :disabled="isCreating"
                class="flex-1 py-2 text-sm font-medium rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') }}
              </button>
              <button
                @click="handleCreate"
                :disabled="isCreating || !isValidProjectName || !createTargetDir.trim()"
                class="flex-1 py-2 text-sm font-medium btn-primary transition-colors disabled:opacity-50"
              >
                {{ isCreating ? t('templates.creating') : t('templates.createProject') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Created Success Quick Access -->
    <Teleport to="body">
      <div v-if="lastCreatedProjectId" class="fixed bottom-6 right-6 z-[80] animate-fade-in">
        <div class="bg-green-600 text-white rounded shadow-sm px-3 py-2 flex items-center gap-2">
          <svg class="w-5 h-5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M5 13l4 4L19 7" />
          </svg>
          <span class="text-sm font-medium">{{ t('templates.createSuccess') }}</span>
          <button
            v-if="lastCreatedProject"
            @click="openProjectWithEngine(lastCreatedProject!)"
            class="ml-1 px-2 py-0.5 text-xs font-medium bg-white/20 hover:bg-white/30 rounded transition-colors"
          >
            {{ t('projects.openWithEngine') }}
          </button>
          <button
            v-if="lastCreatedProject"
            @click="openInFileManager(lastCreatedProject!.path)"
            class="px-2 py-0.5 text-xs font-medium bg-white/20 hover:bg-white/30 rounded transition-colors"
          >
            {{ t('projects.openInFileManager') }}
          </button>
          <button
            @click="lastCreatedProjectId = ''; lastCreatedProject = null"
            class="ml-1 text-green-200 hover:text-white transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </Teleport>

    <!-- Import URL Dialog -->
    <Teleport to="body">
      <div v-if="showImportDialog" class="fixed inset-0 z-[60] flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isImporting && (showImportDialog = false)"></div>
        <div class="dialog-container relative z-10 max-w-md w-full mx-4">
          <div class="p-4">
            <h2 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('templates.importUrl') }}</h2>
            <input
              v-model="importUrl"
              type="url"
              :placeholder="t('templates.importUrlPlaceholder')"
              :disabled="isImporting"
              class="w-full px-3 py-2 text-sm rounded border border-gray-300 dark:border-surface-border bg-white dark:bg-surface-layer text-gray-900 dark:text-content-primary focus:ring-1 focus:ring-primary-500 outline-none disabled:opacity-50"
            />
            <div class="flex gap-2 mt-4">
              <button
                @click="showImportDialog = false"
                :disabled="isImporting"
                class="flex-1 py-2 text-sm font-medium rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') || 'Cancel' }}
              </button>
              <button
                @click="handleImport"
                :disabled="isImporting || !isValidImportUrl || !isOnline"
                class="flex-1 py-2 text-sm font-medium btn-primary transition-colors disabled:opacity-50"
              >
                {{ isImporting ? '...' : !isOnline ? t('common.offlineImportTip') : t('common.import') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Generate From Project Dialog -->
    <Teleport to="body">
      <div v-if="showGenerateFromProjectDialog" class="fixed inset-0 z-[60] flex items-center justify-center">
        <div class="absolute inset-0 bg-black/50" @click="!isGenerating && (showGenerateFromProjectDialog = false)"></div>
        <div class="dialog-container relative z-10 max-w-md w-full mx-4">
          <div class="p-4">
            <h2 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-3">{{ t('templates.generateFromProject') }}</h2>
            <div class="space-y-3">
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.selectProject') }}</label>
                <ProjectSelector v-model="generateProjectId" :projects="projects" :placeholder="t('templates.selectProjectPlaceholder')" />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.templateName') }}</label>
                <input
                  v-model="generateTemplateName"
                  type="text"
                  :placeholder="t('templates.templateNamePlaceholder')"
                  :disabled="isGenerating"
                  class="input-field disabled:opacity-50"
                />
              </div>
              <div>
                <label class="block text-sm font-medium text-gray-700 dark:text-content-secondary mb-1">{{ t('templates.category') }}</label>
                <select
                  v-model="generateCategory"
                  class="select-field"
                >
                  <option value="Custom">{{ t('templates.category.Custom') }}</option>
                  <option value="Starter2D">{{ t('templates.category.Starter2D') }}</option>
                  <option value="Starter3D">{{ t('templates.category.Starter3D') }}</option>
                  <option value="RPG">{{ t('templates.category.RPG') }}</option>
                  <option value="Platformer">{{ t('templates.category.Platformer') }}</option>
                  <option value="Multiplayer">{{ t('templates.category.Multiplayer') }}</option>
                  <option value="Mobile">{{ t('templates.category.Mobile') }}</option>
                </select>
              </div>
            </div>
            <div class="flex gap-2 mt-4">
              <button
                @click="showGenerateFromProjectDialog = false"
                :disabled="isGenerating"
                class="flex-1 py-2 text-sm font-medium rounded border border-gray-300 dark:border-surface-border text-gray-700 dark:text-content-primary hover:bg-gray-50 dark:hover:bg-surface-layer transition-colors disabled:opacity-50"
              >
                {{ t('common.cancel') }}
              </button>
              <button
                @click="handleGenerateFromProject"
                :disabled="isGenerating || !generateProjectId || !generateTemplateName.trim()"
                class="flex-1 py-2 text-sm font-medium btn-primary transition-colors disabled:opacity-50"
              >
                {{ isGenerating ? '...' : t('common.generate') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <ConfirmDialog
      v-model="showDeleteConfirm"
      :title="t('templates.deleteConfirm')"
      :confirm-text="t('common.delete')"
      confirm-color="red"
      @confirm="handleDelete"
      @update:model-value="(v: boolean) => { if (!v) deleteTargetId = '' }"
    />

    <!-- 引擎选择对话框 -->
    <Teleport to="body">
      <div v-if="showEngineSelectDialog && engineSelectProject" class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" @click="closeEngineSelectDialog">
        <div class="dialog-container w-full max-w-md max-h-[80vh] flex flex-col" @click.stop>
          <h3 class="text-sm font-semibold text-gray-900 dark:text-content-primary mb-1">{{ t('projects.openWithEngine') }}</h3>
          <p class="text-sm text-gray-500 dark:text-content-muted mb-4">
            {{ t('projects.openWithEngineDesc') }}
            <span class="font-mono text-xs bg-gray-100 dark:bg-surface-hover px-1.5 py-0.5 rounded ml-1">Godot {{ engineSelectProject.godot_version }}</span>
          </p>
          <div v-if="isLoadingEngines" class="flex-1 flex items-center justify-center py-8">
            <div class="animate-spin rounded-full h-8 w-8 border-2 border-primary-600 border-t-transparent"></div>
          </div>
          <div v-else-if="matchedEngines.length === 0" class="flex-1 py-8 text-center">
            <svg class="mx-auto h-10 w-10 text-gray-400 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="butt" stroke-linejoin="miter" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
            <p class="text-sm font-medium text-gray-700 dark:text-content-secondary">{{ t('projects.noMatchingEngines') }}</p>
            <p class="text-xs text-gray-500 dark:text-content-muted mt-1">{{ t('projects.noMatchingEnginesDesc') }}</p>
          </div>
          <div v-else class="flex-1 overflow-y-auto space-y-2 min-h-0">
            <button
              v-for="me in matchedEngines"
              :key="me.engine.engine_id"
              @click="launchWithEngine(me.engine.engine_id)"
              :disabled="isLaunching"
              :class="[
                'w-full text-left p-3 rounded border transition-colors disabled:opacity-40 disabled:cursor-not-allowed',
                me.engine.engine_id === engineSelectProject?.last_used_engine_id
                  ? 'border-primary-300 dark:border-surface-border bg-primary-50 dark:bg-surface-hover'
                  : 'border-gray-200/60 dark:border-surface-border/40 hover:border-primary-300 dark:hover:border-surface-border hover:bg-primary-50 dark:hover:bg-surface-hover'
              ]"
            >
              <div class="flex items-center justify-between">
                <div class="min-w-0 flex-1">
                  <div class="text-sm font-medium text-gray-900 dark:text-content-primary truncate flex items-center gap-1.5">
                    {{ me.engine.name }}
                    <span v-if="me.engine.engine_id === engineSelectProject?.last_used_engine_id" class="text-xs text-primary-600 dark:text-brand-primary font-normal">{{ t('projects.lastUsedEngine') }}</span>
                  </div>
                  <div class="text-xs text-gray-500 dark:text-content-muted mt-0.5 font-mono flex items-center gap-1.5">v{{ me.engine.version }}<span v-if="me.engine.is_mono" class="text-[10px] px-1 py-0.5 rounded bg-purple-100 dark:bg-surface-hover text-purple-700 dark:text-content-secondary font-sans font-medium">{{ t('projects.monoLabel') }}</span></div>
                </div>
                <span :class="['text-xs px-2 py-0.5 rounded-full font-medium ml-2 flex-shrink-0', getMatchLevelClass(me.match_level)]" :title="getMatchLevelDesc(me.match_level)">{{ getMatchLevelLabel(me.match_level) }}</span>
              </div>
            </button>
          </div>
          <div class="flex justify-end mt-3 pt-2 border-t border-gray-200/60 dark:border-surface-border/40">
            <button @click="closeEngineSelectDialog" class="btn-secondary">{{ t('common.cancel') }}</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Export Template Dialog -->
    <TemplateExportDialog
      v-model="showExportDialog"
      :template="exportTemplate"
      @exported="() => { loadTemplates() }"
    />

    <!-- Import Template File Dialog -->
    <TemplateImportDialog
      v-model="showImportFileDialog"
      @imported="() => { loadTemplates() }"
    />

    <!-- Keypair Manager -->
    <KeypairManager
      v-model="showKeypairManager"
    />

    <ContextMenu
      :visible="templateContextMenu.visible.value"
      :x="templateContextMenu.x.value"
      :y="templateContextMenu.y.value"
      :items="templateContextMenu.items.value"
      @close="templateContextMenu.close()"
    />

  </div>
</template>
