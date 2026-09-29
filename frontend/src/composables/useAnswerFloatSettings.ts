import { computed, reactive, watch } from 'vue'

const STORAGE_KEY = 'exameow-answer-float-settings'
const defaults = { backgroundTransparency: 0, textTransparency: 0, fontSize: 13, closeClicks: 3, closeSeconds: 3, shortcut: 'CommandOrControl+Shift+F8' }

function bounded(value: unknown, fallback: number, min: number, max: number): number {
  return typeof value === 'number' && Number.isFinite(value)
    ? Math.min(max, Math.max(min, Math.round(value)))
    : fallback
}

export function useAnswerFloatSettings() {
  let saved: Partial<typeof defaults> = {}
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}')
    if (parsed && typeof parsed === 'object') saved = parsed
  } catch { /* Fall back to readable defaults when storage is unavailable or invalid. */ }

  const settings = reactive({
    backgroundTransparency: bounded(saved.backgroundTransparency, 0, 0, 100),
    textTransparency: bounded(saved.textTransparency, 0, 0, 100),
    fontSize: bounded(saved.fontSize, 13, 10, 36),
    closeClicks: bounded(saved.closeClicks, 3, 2, 5),
    closeSeconds: bounded(saved.closeSeconds, 3, 1, 10),
    shortcut: typeof saved.shortcut === 'string' && saved.shortcut.length > 0 && saved.shortcut.length <= 128
      ? saved.shortcut : defaults.shortcut,
  })
  watch(settings, () => {
    try { localStorage.setItem(STORAGE_KEY, JSON.stringify(settings)) } catch { /* Session still works. */ }
  })

  const style = computed(() => ({
    '--background-opacity': String(1 - settings.backgroundTransparency / 100),
    '--text-opacity': String(1 - settings.textTransparency / 100),
    '--answer-font-size': `${settings.fontSize}px`,
  }))

  return { settings, style, reset: () => Object.assign(settings, { backgroundTransparency: 0, textTransparency: 0, fontSize: 13 }) }
}
