import { useVisibility } from '@/composables/useVisibility'

const { isVisible, show: showLanguageDialog, hide: hideLanguageDialog } = useVisibility('languageDialog')

export function useLanguageDialog() {
  return { isVisible, showLanguageDialog, hideLanguageDialog }
}
