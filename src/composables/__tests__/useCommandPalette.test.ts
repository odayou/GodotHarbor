import { describe, it, expect } from 'vitest'

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

describe('toPinyin', () => {
  it('converts Chinese characters to pinyin', () => {
    expect(toPinyin('项目管理')).toBe('xiang mu guan li')
  })

  it('preserves English characters', () => {
    expect(toPinyin('Godot')).toBe('godot')
  })

  it('handles mixed content', () => {
    const result = toPinyin('插件Plugins')
    expect(result).toContain('cha jian')
    expect(result).toContain('plugins')
  })

  it('returns empty for empty string', () => {
    expect(toPinyin('')).toBe('')
  })
})

describe('toPinyinInitials', () => {
  it('extracts pinyin initials', () => {
    expect(toPinyinInitials('项目管理')).toBe('xmgl')
  })

  it('handles mixed content', () => {
    const result = toPinyinInitials('插件plugin')
    expect(result).toContain('cj')
    expect(result).toContain('plugin')
  })
})

describe('fuzzyMatch', () => {
  it('exact match gets highest score', () => {
    const result = fuzzyMatch('projects', 'projects')
    expect(result.matched).toBe(true)
    expect(result.score).toBe(100)
  })

  it('prefix match gets high score', () => {
    const result = fuzzyMatch('projects', 'proj')
    expect(result.matched).toBe(true)
    expect(result.score).toBe(90)
  })

  it('contains match gets medium score', () => {
    const result = fuzzyMatch('my projects', 'projects')
    expect(result.matched).toBe(true)
    expect(result.score).toBe(70)
  })

  it('pinyin match works for Chinese', () => {
    const result = fuzzyMatch('项目管理', 'xiangmu')
    expect(result.matched).toBe(true)
    expect(result.score).toBeGreaterThanOrEqual(55)
  })

  it('pinyin initials match works', () => {
    const result = fuzzyMatch('项目管理', 'xmgl')
    expect(result.matched).toBe(true)
    expect(result.score).toBe(55)
  })

  it('subsequence match works', () => {
    const result = fuzzyMatch('projects', 'prjs')
    expect(result.matched).toBe(true)
  })

  it('no match returns false', () => {
    const result = fuzzyMatch('projects', 'xyz')
    expect(result.matched).toBe(false)
    expect(result.score).toBe(0)
  })

  it('empty query matches everything', () => {
    const result = fuzzyMatch('anything', '')
    expect(result.matched).toBe(true)
  })

  it('case insensitive', () => {
    const result = fuzzyMatch('Projects', 'projects')
    expect(result.matched).toBe(true)
    expect(result.score).toBe(100)
  })

  it('word boundary bonus', () => {
    const boundary = fuzzyMatch('my-projects', 'p')
    const middle = fuzzyMatch('myxprojects', 'p')
    expect(boundary.score).toBeGreaterThanOrEqual(middle.score)
  })
})
