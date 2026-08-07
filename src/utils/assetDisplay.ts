export interface BadgeStyle {
  text: string
  class: string
}

export interface RatingStars {
  full: number
  half: boolean
  empty: number
  value: number
}

/** 计算 5 星展示所需的 full/half/empty 数量，兼容字符串或数字入参。 */
export function getRatingStars(rating: string | number): RatingStars {
  const r = typeof rating === 'number' ? rating : (parseFloat(rating) || 0)
  const full = Math.floor(r)
  const half = r - full >= 0.5
  const empty = 5 - full - (half ? 1 : 0)
  return { full, half, empty, value: r }
}

/** Asset Library 兼容性徽章（Godot 3 / 4 / 未知）。 */
export function getCompatibilityBadge(godotVersion: string | null | undefined): BadgeStyle | null {
  const gv = (godotVersion || '').toLowerCase()
  if (gv.includes('4.') || gv.includes('4.x')) {
    return { text: 'Godot 4', class: 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400' }
  }
  if (gv.includes('3.') || gv.includes('3.x')) {
    return { text: 'Godot 3', class: 'bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400' }
  }
  return null
}

/** 支持等级徽章；text 本地化依赖外部传入的 i18n 翻译函数。 */
export function getSupportBadge(level: string, t: (key: string) => string): BadgeStyle | null {
  switch (level) {
    case 'official': return { text: t('assetLibrary.supportOfficial'), class: 'bg-blue-100 text-blue-800 dark:bg-surface-hover dark:text-brand-primary' }
    case 'featured': return { text: t('assetLibrary.supportFeatured'), class: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400' }
    case 'community': return { text: t('assetLibrary.supportCommunity'), class: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400' }
    case 'testing': return { text: t('assetLibrary.supportTesting'), class: 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400' }
    default: return null
  }
}