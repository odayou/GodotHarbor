import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores'
import { usePluginStore } from '@/stores'
import { useTheme } from '@/composables/useTheme'
import { useSidebar } from '@/composables/useSidebar'

export interface SearchItem {
  id: string
  label: string
  category: 'project' | 'plugin' | 'engine' | 'setting' | 'command' | 'navigation'
  icon: string
  keywords: string
  action: () => void
  shortcutKey?: string
}

const isOpen = ref(false)
const query = ref('')
const selectedIndex = ref(0)

let globalListenerRegistered = false
function ensureGlobalListener() {
  if (globalListenerRegistered) return
  globalListenerRegistered = true
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    if (e.key.toLowerCase() === 'k' && (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey) {
      e.preventDefault()
      if (isOpen.value) {
        isOpen.value = false
        query.value = ''
        selectedIndex.value = 0
      } else {
        isOpen.value = true
        query.value = ''
        selectedIndex.value = 0
      }
    }
  })
}
ensureGlobalListener()

const PINYIN_MAP: Record<string, string> = {
  '首': 'shou', '页': 'ye', '项': 'xiang', '目': 'mu', '管': 'guan', '理': 'li',
  '插': 'cha', '件': 'jian', '仓': 'cang', '库': 'ku', '绑': 'bang', '定': 'ding',
  '引': 'yin', '擎': 'qing', '路': 'lu', '线': 'xian', '图': 'tu', '关': 'guan',
  '于': 'yu', '设': 'she', '置': 'zhi', '扫': 'sao', '描': 'miao', '添': 'tian',
  '加': 'jia', '删': 'shan', '除': 'chu', '导': 'dao', '入': 'ru', '出': 'chu',
  '搜': 'sou', '索': 'suo', '全': 'quan', '局': 'ju', '命': 'ming', '令': 'ling',
  '面': 'mian', '板': 'ban', '切': 'qie', '换': 'huan', '主': 'zhu', '题': 'ti',
  '深': 'shen', '色': 'se', '浅': 'qian', '模': 'mo', '式': 'shi', '侧': 'ce',
  '边': 'bian', '栏': 'lan', '折': 'zhe', '叠': 'die', '展': 'zhan', '开': 'kai',
  '暗': 'an', '亮': 'liang', '跟': 'gen', '随': 'sui', '系': 'xi', '统': 'tong',
  '启': 'qi', '动': 'dong', '自': 'zi', '发': 'fa', '现': 'xian', '注': 'zhu',
  '册': 'ce', '备': 'bei', '份': 'fen', '恢': 'hui', '复': 'fu',
  '数': 'shu', '据': 'ju', '保': 'bao', '存': 'cun', '取': 'qu', '消': 'xiao',
  '确': 'que', '认': 'ren', '语': 'yu', '言': 'yan', '挂': 'gua', '载': 'zai',
  '策': 'ce', '略': 'lue', '符': 'fu', '号': 'hao', '链': 'lian', '接': 'jie',
  '制': 'zhi', '应': 'ying', '用': 'yong', '变': 'bian', '更': 'geng',
  '版': 'ban', '本': 'ben', '来': 'lai', '源': 'yuan', '收': 'shou', '藏': 'cang',
  '详': 'xiang', '情': 'qing', '未': 'wei', '知': 'zhi', '作': 'zuo', '者': 'zhe',
  '述': 'shu', '状': 'zhuang', '态': 'tai', '就': 'jiu', '绪': 'xu',
  '警': 'jing', '告': 'gao', '错': 'cuo', '误': 'wu', '冲': 'chong', '突': 'tu',
  '缺': 'que', '失': 'shi', '径': 'jing', '名': 'ming', '称': 'cheng',
  '分': 'fen', '组': 'zu', '参': 'can',
}

function toPinyin(text: string): string {
  let result = ''
  for (const char of text) {
    if (PINYIN_MAP[char]) {
      result += PINYIN_MAP[char] + ' '
    } else if (/[a-zA-Z0-9]/.test(char)) {
      result += char.toLowerCase()
    } else {
      result += ' '
    }
  }
  return result.trim()
}

function toPinyinInitials(text: string): string {
  let result = ''
  for (const char of text) {
    if (PINYIN_MAP[char]) {
      result += PINYIN_MAP[char][0]
    } else if (/[a-zA-Z0-9]/.test(char)) {
      result += char.toLowerCase()
    }
  }
  return result
}

function fuzzyMatch(text: string, searchQuery: string): { matched: boolean; score: number } {
  const lowerText = text.toLowerCase()
  const lowerQuery = searchQuery.toLowerCase()

  if (lowerText === lowerQuery) return { matched: true, score: 100 }
  if (lowerText.startsWith(lowerQuery)) return { matched: true, score: 90 }
  if (lowerText.includes(lowerQuery)) return { matched: true, score: 70 }

  const pinyinText = toPinyin(text)
  const pinyinInitials = toPinyinInitials(text)

  if (pinyinText.includes(lowerQuery)) return { matched: true, score: 60 }
  if (pinyinInitials.includes(lowerQuery)) return { matched: true, score: 55 }

  let queryIdx = 0
  let score = 0
  let lastMatchIdx = -1

  for (let i = 0; i < lowerText.length && queryIdx < lowerQuery.length; i++) {
    if (lowerText[i] === lowerQuery[queryIdx]) {
      score += 10
      if (lastMatchIdx === -1 || i === lastMatchIdx + 1) {
        score += 5
      }
      if (i === 0 || lowerText[i - 1] === ' ' || lowerText[i - 1] === '-' || lowerText[i - 1] === '_') {
        score += 8
      }
      lastMatchIdx = i
      queryIdx++
    }
  }

  if (queryIdx === lowerQuery.length) {
    return { matched: true, score }
  }

  queryIdx = 0
  lastMatchIdx = -1
  score = 0
  for (let i = 0; i < pinyinText.length && queryIdx < lowerQuery.length; i++) {
    if (pinyinText[i] === lowerQuery[queryIdx]) {
      score += 8
      if (lastMatchIdx === -1 || i === lastMatchIdx + 1) {
        score += 3
      }
      lastMatchIdx = i
      queryIdx++
    }
  }

  if (queryIdx === lowerQuery.length) {
    return { matched: true, score }
  }

  return { matched: false, score: 0 }
}

export function useCommandPalette() {
  const router = useRouter()
  const projectStore = useProjectStore()
  const pluginStore = usePluginStore()
  const { currentTheme, setTheme } = useTheme()
  const { toggleSidebar } = useSidebar()
  const { t, locale } = useI18n()

  const allItems = computed<SearchItem[]>(() => {
    const items: SearchItem[] = []
    const shortcutKeys = ['1', '2', '3', '4', '5', '6', '7', '8', '9']

    items.push(
      {
        id: 'nav-home',
        label: t('nav.home'),
        category: 'navigation',
        icon: 'home',
        keywords: `${t('nav.home')} home`,
        action: () => { router.push('/'); closePalette() },
        shortcutKey: shortcutKeys[0]
      },
      {
        id: 'nav-projects',
        label: t('nav.projects'),
        category: 'navigation',
        icon: 'folder',
        keywords: `${t('nav.projects')} projects`,
        action: () => { router.push('/projects'); closePalette() },
        shortcutKey: shortcutKeys[1]
      },
      {
        id: 'nav-plugins',
        label: t('nav.pluginsNav'),
        category: 'navigation',
        icon: 'puzzle',
        keywords: `${t('nav.pluginsNav')} plugins`,
        action: () => { router.push('/plugins'); closePalette() },
        shortcutKey: shortcutKeys[2]
      },
      {
        id: 'nav-linker',
        label: t('nav.linkerNav'),
        category: 'navigation',
        icon: 'link',
        keywords: `${t('nav.linkerNav')} linker bind mount`,
        action: () => { router.push('/plugins?tab=bindings'); closePalette() },
        shortcutKey: shortcutKeys[3]
      },
      {
        id: 'nav-engines',
        label: t('nav.enginesNav'),
        category: 'navigation',
        icon: 'engine',
        keywords: `${t('nav.enginesNav')} engines`,
        action: () => { router.push('/engines'); closePalette() },
        shortcutKey: shortcutKeys[4]
      },
      {
        id: 'nav-updates',
        label: t('nav.updates'),
        category: 'navigation',
        icon: 'updates',
        keywords: `${t('nav.updates')} updates`,
        action: () => { router.push('/updates'); closePalette() },
        shortcutKey: shortcutKeys[5]
      },
      {
        id: 'nav-settings',
        label: t('nav.settingsNav'),
        category: 'navigation',
        icon: 'settings',
        keywords: `${t('nav.settingsNav')} settings`,
        action: () => { router.push('/settings'); closePalette() },
        shortcutKey: shortcutKeys[6]
      },
      {
        id: 'nav-about',
        label: t('nav.about'),
        category: 'navigation',
        icon: 'about',
        keywords: `${t('nav.about')} about`,
        action: () => { router.push('/about'); closePalette() },
        shortcutKey: shortcutKeys[7]
      },
      {
        id: 'cmd-toggle-theme',
        label: t('commandPalette.toggleTheme'),
        category: 'command',
        icon: 'theme',
        keywords: `${t('commandPalette.toggleTheme')} theme dark light`,
        action: () => {
          setTheme(currentTheme.value === 'dark' ? 'light' : 'dark')
          closePalette()
        }
      },
      {
        id: 'cmd-toggle-sidebar',
        label: t('commandPalette.toggleSidebar'),
        category: 'command',
        icon: 'sidebar',
        keywords: `${t('commandPalette.toggleSidebar')} sidebar`,
        action: () => {
          toggleSidebar()
          closePalette()
        }
      },
      {
        id: 'cmd-scan-projects',
        label: t('commandPalette.scanProjects'),
        category: 'command',
        icon: 'scan',
        keywords: `${t('commandPalette.scanProjects')} scan`,
        action: () => {
          router.push('/projects?action=scan')
          closePalette()
        }
      },
      {
        id: 'cmd-import-plugin',
        label: t('commandPalette.importPlugin'),
        category: 'command',
        icon: 'import',
        keywords: `${t('commandPalette.importPlugin')} import plugin`,
        action: () => {
          router.push('/plugins?action=import')
          closePalette()
        }
      },
      {
        id: 'cmd-register-engine',
        label: t('commandPalette.registerEngine'),
        category: 'command',
        icon: 'engine',
        keywords: `${t('commandPalette.registerEngine')} register engine`,
        action: () => {
          router.push('/engines?action=register')
          closePalette()
        }
      },
      {
        id: 'cmd-toggle-language',
        label: locale.value === 'zh-CN' ? t('commandPalette.switchToEnglish') : t('commandPalette.switchToChinese'),
        category: 'command',
        icon: 'language',
        keywords: `language english chinese ${t('commandPalette.chineseEnglish')} ${t('commandPalette.switchLanguage')}`,
        action: () => {
          locale.value = locale.value === 'zh-CN' ? 'en' : 'zh-CN'
          closePalette()
        }
      }
    )

    projectStore.projects.forEach(p => {
      items.push({
        id: `project-${p.project_id}`,
        label: p.name,
        category: 'project',
        icon: 'folder',
        keywords: `${p.name} ${p.path} ${p.godot_version} ${p.group || ''}`,
        action: () => { router.push('/projects'); closePalette() }
      })
    })

    pluginStore.plugins.forEach(p => {
      const versionStr = p.versions.length > 0 ? p.versions[0].version : ''
      items.push({
        id: `plugin-${p.plugin_id}`,
        label: p.name,
        category: 'plugin',
        icon: 'puzzle',
        keywords: `${p.name} ${p.description} ${p.author} ${versionStr}`,
        action: () => { router.push('/plugins'); closePalette() }
      })
    })

    return items
  })

  const filteredItems = computed(() => {
    if (!query.value.trim()) {
      return allItems.value.slice(0, 30)
    }

    const results: { item: SearchItem; score: number }[] = []

    for (const item of allItems.value) {
      const searchText = `${item.label} ${item.keywords}`
      const { matched, score } = fuzzyMatch(searchText, query.value)
      if (matched) {
        results.push({ item, score })
      }
    }

    results.sort((a, b) => b.score - a.score)
    return results.map(r => r.item)
  })

  const groupedResults = computed(() => {
    const groups: { category: SearchItem['category']; items: SearchItem[] }[] = []
    const categoryOrder: SearchItem['category'][] = ['navigation', 'command', 'project', 'plugin', 'engine', 'setting']
    const categoryMap = new Map<SearchItem['category'], SearchItem[]>()

    for (const item of filteredItems.value) {
      const existing = categoryMap.get(item.category)
      if (existing) {
        existing.push(item)
      } else {
        categoryMap.set(item.category, [item])
      }
    }

    for (const cat of categoryOrder) {
      const items = categoryMap.get(cat)
      if (items && items.length > 0) {
        groups.push({ category: cat, items })
      }
    }

    return groups
  })

  watch(filteredItems, () => {
    selectedIndex.value = 0
  })

  function openPalette() {
    isOpen.value = true
    query.value = ''
    selectedIndex.value = 0
    loadSearchData()
  }

  function closePalette() {
    isOpen.value = false
    query.value = ''
    selectedIndex.value = 0
  }

  async function loadSearchData() {
    const promises: Promise<void>[] = []
    if (projectStore.projects.length === 0) {
      promises.push(projectStore.loadProjects().then(() => {}).catch(() => {}))
    }
    if (pluginStore.plugins.length === 0) {
      promises.push(pluginStore.loadPlugins().then(() => {}).catch(() => {}))
    }
    await Promise.allSettled(promises)
  }

  function selectItem(item: SearchItem) {
    item.action()
  }

  function moveSelection(delta: number) {
    const total = filteredItems.value.length
    if (total === 0) return
    selectedIndex.value = ((selectedIndex.value + delta) % total + total) % total
  }

  function selectCurrentItem() {
    const item = filteredItems.value[selectedIndex.value]
    if (item) {
      selectItem(item)
    }
  }

  function selectByShortcutKey(key: string): boolean {
    const item = filteredItems.value.find(i => i.shortcutKey === key)
    if (item) {
      selectItem(item)
      return true
    }
    return false
  }

  return {
    isOpen,
    query,
    selectedIndex,
    filteredItems,
    groupedResults,
    allItems,
    openPalette,
    closePalette,
    selectItem,
    moveSelection,
    selectCurrentItem,
    selectByShortcutKey,
    t
  }
}
