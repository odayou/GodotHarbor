import { useVisibility } from '@/composables/useVisibility'

const { isVisible, show: showOnboarding, hide: hideOnboarding } = useVisibility('onboarding')

export function useOnboarding() {
  return { isVisible, showOnboarding, hideOnboarding }
}
