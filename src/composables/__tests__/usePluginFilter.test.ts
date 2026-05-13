import { describe, it, expect } from 'vitest'

interface Plugin {
  plugin_id: string
  name: string
  description: string
  author: string
  compatibility: string
  is_favorite: boolean
  source: { source_type: string; url: string }
  versions: { version: string }[]
}

function filterPlugins(
  plugins: Plugin[],
  searchQuery: string,
  filterCompatibility: string,
  filterSource: string,
  showFavoritesOnly: boolean,
  showOnlyDuplicates: boolean
): Plugin[] {
  return plugins.filter(plugin => {
    const matchesSearch = searchQuery === '' ||
      plugin.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      plugin.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      plugin.author.toLowerCase().includes(searchQuery.toLowerCase())

    const matchesCompatibility = filterCompatibility === 'all' ||
      plugin.compatibility === filterCompatibility

    const matchesSource = filterSource === 'all' ||
      plugin.source.source_type === filterSource

    const matchesFavorite = !showFavoritesOnly || plugin.is_favorite === true

    const matchesDuplicate = !showOnlyDuplicates || plugins.filter(p => p.name === plugin.name && p.plugin_id !== plugin.plugin_id).length > 0

    return matchesSearch && matchesCompatibility && matchesSource && matchesFavorite && matchesDuplicate
  })
}

function paginatePlugins(filtered: Plugin[], displayLimit: number): Plugin[] {
  if (filtered.length <= displayLimit) return filtered
  return filtered.slice(0, displayLimit)
}

const mockPlugins: Plugin[] = [
  {
    plugin_id: '1', name: 'GodotSteam', description: 'Steam integration', author: 'Coagu',
    compatibility: 'Godot4', is_favorite: true, source: { source_type: 'Git', url: 'https://github.com/test' },
    versions: [{ version: '1.0' }]
  },
  {
    plugin_id: '2', name: 'GodotSteam', description: 'Steam integration fork', author: 'Other',
    compatibility: 'Godot3', is_favorite: false, source: { source_type: 'Local', url: '/local/path' },
    versions: [{ version: '0.9' }]
  },
  {
    plugin_id: '3', name: 'Dialogic', description: 'Dialog system', author: 'Jowan',
    compatibility: 'Godot4', is_favorite: false, source: { source_type: 'AssetLibrary', url: '' },
    versions: [{ version: '2.0' }]
  },
  {
    plugin_id: '4', name: 'PhantomCamera', description: 'Camera plugin', author: 'Marcel',
    compatibility: 'Both', is_favorite: true, source: { source_type: 'Git', url: 'https://github.com/test2' },
    versions: [{ version: '0.5' }]
  },
]

describe('filterPlugins', () => {
  it('returns all plugins with default filters', () => {
    const result = filterPlugins(mockPlugins, '', 'all', 'all', false, false)
    expect(result).toHaveLength(4)
  })

  it('filters by search query - name', () => {
    const result = filterPlugins(mockPlugins, 'steam', 'all', 'all', false, false)
    expect(result).toHaveLength(2)
    expect(result.every(p => p.name.toLowerCase().includes('steam'))).toBe(true)
  })

  it('filters by search query - description', () => {
    const result = filterPlugins(mockPlugins, 'dialog', 'all', 'all', false, false)
    expect(result).toHaveLength(1)
    expect(result[0].name).toBe('Dialogic')
  })

  it('filters by search query - author', () => {
    const result = filterPlugins(mockPlugins, 'marcel', 'all', 'all', false, false)
    expect(result).toHaveLength(1)
    expect(result[0].name).toBe('PhantomCamera')
  })

  it('filters by compatibility', () => {
    const result = filterPlugins(mockPlugins, '', 'Godot4', 'all', false, false)
    expect(result).toHaveLength(2)
    expect(result.every(p => p.compatibility === 'Godot4')).toBe(true)
  })

  it('filters by source type', () => {
    const result = filterPlugins(mockPlugins, '', 'all', 'Git', false, false)
    expect(result).toHaveLength(2)
    expect(result.every(p => p.source.source_type === 'Git')).toBe(true)
  })

  it('filters favorites only', () => {
    const result = filterPlugins(mockPlugins, '', 'all', 'all', true, false)
    expect(result).toHaveLength(2)
    expect(result.every(p => p.is_favorite)).toBe(true)
  })

  it('filters duplicates only', () => {
    const result = filterPlugins(mockPlugins, '', 'all', 'all', false, true)
    expect(result).toHaveLength(2)
    expect(result.every(p => p.name === 'GodotSteam')).toBe(true)
  })

  it('combines multiple filters', () => {
    const result = filterPlugins(mockPlugins, 'steam', 'Godot4', 'all', false, false)
    expect(result).toHaveLength(1)
    expect(result[0].plugin_id).toBe('1')
  })

  it('returns empty for no matches', () => {
    const result = filterPlugins(mockPlugins, 'nonexistent', 'all', 'all', false, false)
    expect(result).toHaveLength(0)
  })

  it('case insensitive search', () => {
    const result = filterPlugins(mockPlugins, 'GODOTSTEAM', 'all', 'all', false, false)
    expect(result).toHaveLength(2)
  })
})

describe('paginatePlugins', () => {
  it('returns all if under limit', () => {
    const result = paginatePlugins(mockPlugins, 50)
    expect(result).toHaveLength(4)
  })

  it('truncates if over limit', () => {
    const result = paginatePlugins(mockPlugins, 2)
    expect(result).toHaveLength(2)
  })

  it('handles empty array', () => {
    const result = paginatePlugins([], 50)
    expect(result).toHaveLength(0)
  })
})
