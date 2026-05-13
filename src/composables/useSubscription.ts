import { ref, computed } from 'vue'

export type Tier = 'free' | 'cloud' | 'marketplace'

const STORAGE_KEY = 'godot-harbor-tier'

const currentTier = ref<Tier>(loadTier())

function loadTier(): Tier {
  try {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved === 'cloud' || saved === 'marketplace') return saved
  } catch {}
  return 'free'
}

export function useSubscription() {
  const tier = computed(() => currentTier.value)

  const isFree = computed(() => currentTier.value === 'free')
  const hasCloud = computed(() => currentTier.value === 'cloud' || currentTier.value === 'marketplace')
  const hasMarketplace = computed(() => currentTier.value === 'marketplace')

  function setTier(newTier: Tier) {
    currentTier.value = newTier
    try {
      localStorage.setItem(STORAGE_KEY, newTier)
    } catch {}
  }

  function resetTier() {
    setTier('free')
  }

  return {
    tier,
    isFree,
    hasCloud,
    hasMarketplace,
    setTier,
    resetTier,
  }
}
