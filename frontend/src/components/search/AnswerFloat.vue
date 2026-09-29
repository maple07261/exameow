<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useScreenRecordStore } from '@/stores/screenRecord'
import { useAnswerFloatSettings } from '@/composables/useAnswerFloatSettings'
import type { Window } from '@tauri-apps/api/window'
import { useI18nStore } from '@/stores/i18n'
import {
  AdjustmentsHorizontalIcon,
  ArrowsPointingOutIcon,
  CheckIcon,
  MagnifyingGlassIcon,
  PauseCircleIcon,
  VideoCameraIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
import { PlayIcon } from '@heroicons/vue/24/solid'

const store = useScreenRecordStore()
const i18n = useI18nStore()

const { settings, style: appearanceStyle, reset: resetAppearance } = useAnswerFloatSettings()
const showAppearance = ref(false)
const topmostError = ref(false)
const isTopmost = ref(true)
const shortcutDraft = ref(settings.shortcut)
const shortcutError = ref('')
const shortcutReady = ref(false)
const shortcutBusy = ref(false)
const layerBusy = ref(false)
const closeAttempts = ref(0)
const closeBlocked = ref(false)
let closeDeadline = 0
let closeTimer: ReturnType<typeof setTimeout> | undefined
let closing = false

function resetClose() {
  clearTimeout(closeTimer)
  closeAttempts.value = 0
  closeDeadline = 0
  closeBlocked.value = false
}
watch(() => [settings.closeClicks, settings.closeSeconds], resetClose)
const adjusting = ref(true)
const hasBegun = ref(false)
const unlistenFns: Array<() => void> = []

let win: Window | null = null
let ctl: {
  initFloat: () => Promise<void>
  adjust: () => Promise<void>
  stop: () => Promise<void>
} | null = null

onMounted(async () => {
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  const { listen } = await import('@tauri-apps/api/event')
  win = getCurrentWindow()
  unlistenFns.push(await listen<boolean>('answer-float:layer', (event) => {
    isTopmost.value = event.payload
    topmostError.value = false
  }))
  unlistenFns.push(await listen<string>('answer-float:control-error', (event) => {
    topmostError.value = true
    console.warn('[answer-float]', event.payload)
  }))
  unlistenFns.push(await win.onCloseRequested((event) => {
    event.preventDefault()
    closeBlocked.value = true
  }))
  await saveShortcut()

  const { useScreenRecord } = await import('@/composables/useScreenRecord')
  ctl = useScreenRecord()
  await ctl.initFloat()

  unlistenFns.push(await listen('screen-record:begin', () => {
    adjusting.value = false
    hasBegun.value = true
  }))
})

onUnmounted(() => {
  resetClose()
  for (const fn of unlistenFns) fn()
})

async function setLayer(topmost: boolean) {
  if (layerBusy.value) return
  layerBusy.value = true
  try {
    isTopmost.value = await invoke<boolean>('set_answer_float_topmost', { topmost })
    topmostError.value = false
  } catch (error) {
    topmostError.value = true
    console.warn('[answer-float] Could not change window layer:', error)
  } finally {
    layerBusy.value = false
  }
}

async function restoreTopmost() { await setLayer(true) }

async function saveShortcut() {
  if (shortcutBusy.value) return
  shortcutBusy.value = true
  shortcutError.value = ''
  try {
    const value = await invoke<string>('configure_answer_float_shortcut', { shortcut: shortcutDraft.value })
    settings.shortcut = value
    shortcutDraft.value = value
    shortcutReady.value = true
  } catch (error) {
    shortcutError.value = i18n.t('answerFloatShortcutFailed')
    console.warn('[answer-float] Shortcut registration failed:', error)
  } finally {
    shortcutBusy.value = false
  }
}

function captureShortcut(event: KeyboardEvent) {
  if (event.key === 'Tab') return
  event.preventDefault()
  if (event.repeat || ['Control', 'Shift', 'Alt', 'Meta'].includes(event.key)) return
  const modifiers: string[] = []
  if (event.ctrlKey) modifiers.push('Control')
  if (event.metaKey) modifiers.push('Super')
  if (event.altKey) modifiers.push('Alt')
  if (event.shiftKey) modifiers.push('Shift')
  if (!event.ctrlKey && !event.metaKey && !event.altKey) {
    shortcutError.value = i18n.t('answerFloatShortcutModifier')
    return
  }
  const key = event.code.replace(/^Key/, '').replace(/^Digit/, '')
  if (!/^(?:[A-Z0-9]|F(?:[1-9]|1[0-9]|2[0-4])|Space|ArrowUp|ArrowDown|ArrowLeft|ArrowRight|Home|End|PageUp|PageDown|Insert|Delete)$/.test(key)) return
  shortcutDraft.value = [...modifiers, key].join('+')
  shortcutError.value = ''
}

function preventRepeatedKey(event: KeyboardEvent) {
  if (event.repeat) event.preventDefault()
}

function onDragArea(e: MouseEvent) {
  if (e.button !== 0 || !win) return
  win.startDragging().catch(() => {})
}

async function handleAdjust() {
  adjusting.value = true
  await ctl?.adjust()
}

async function handleExit() {
  if (closing || !ctl) return
  const now = performance.now()
  if (now >= closeDeadline) {
    resetClose()
    closeDeadline = now + settings.closeSeconds * 1000
    closeTimer = setTimeout(resetClose, settings.closeSeconds * 1000)
  }
  closeAttempts.value += 1
  if (closeAttempts.value < settings.closeClicks) return
  resetClose()
  closing = true
  try { await ctl.stop() } finally { closing = false }
}

async function handleBegin() {
  hasBegun.value = true
  const { emit } = await import('@tauri-apps/api/event')
  await emit('screen-record:request-begin')
}

function isCorrect(idx: number): boolean {
  const r = store.currentResult
  if (!r) return false
  const q = r.question
  if (q.type !== 'single_choice' && q.type !== 'multi_choice') return false
  const letters = (q.answer ?? '').trim().toUpperCase().replace(/[^A-H]/g, '')
  return letters.includes(String.fromCharCode(65 + idx))
}
</script>

<template>
  <div class="w-full h-full select-none">
    <div class="float-card w-full h-full flex flex-col" :style="appearanceStyle">
      <div
        class="shrink-0 cursor-grab active:cursor-grabbing"
        @mousedown="onDragArea"
      >
        <div class="flex justify-center pt-2">
          <div class="float-grip" />
        </div>
        <div class="flex items-center justify-between pl-3.5 pr-2 py-1.5">
          <div class="flex items-center gap-1.5 min-w-0">
            <VideoCameraIcon class="w-4 h-4 shrink-0" style="color: var(--accent);" />
            <span class="float-title text-[13px] font-semibold truncate" style="color: var(--fg);">
              {{ i18n.t('searchModeScreenRecord') }}
            </span>
          </div>
          <div class="float-actions flex items-center gap-1.5 shrink-0">
            <button
              class="float-btn"
              @mousedown.stop
              @click="showAppearance = !showAppearance"
              :title="i18n.t('answerFloatAppearance')"
              :aria-label="i18n.t('answerFloatAppearance')"
              :aria-expanded="showAppearance"
              aria-controls="answer-appearance"
            >
              <AdjustmentsHorizontalIcon class="w-4 h-4" />
            </button>
            <button
              class="float-btn"
              @mousedown.stop
              @click="handleAdjust"
              :title="i18n.t('searchScreenRecordAdjust')"
            >
              <ArrowsPointingOutIcon class="w-4 h-4" />
            </button>
            <button
              class="float-btn float-btn-danger"
              @mousedown.stop
              @click="handleExit"
              @keydown="preventRepeatedKey"
              :title="i18n.t('searchScreenRecordExit')"
            >
              <XMarkIcon class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      <div v-if="showAppearance" id="answer-appearance" class="float-settings shrink-0">
        <div class="flex items-center justify-between gap-2 mb-2">
          <strong>{{ i18n.t('answerFloatAppearance') }}</strong>
          <button class="underline cursor-pointer" @click="resetAppearance">{{ i18n.t('answerFloatReset') }}</button>
        </div>
        <label class="float-setting">
          <span>{{ i18n.t('answerFloatBackground') }} <output>{{ settings.backgroundTransparency }}%</output></span>
          <input v-model.number="settings.backgroundTransparency" type="range" min="0" max="100" step="1" />
        </label>
        <label class="float-setting">
          <span>{{ i18n.t('answerFloatText') }} <output>{{ settings.textTransparency }}%</output></span>
          <input v-model.number="settings.textTransparency" type="range" min="0" max="100" step="1" />
        </label>
        <label class="float-setting">
          <span>{{ i18n.t('answerFloatFontSize') }} <output>{{ settings.fontSize }} px</output></span>
          <input v-model.number="settings.fontSize" type="range" min="10" max="36" step="1" />
        </label>
        <label class="float-setting">
          <span>{{ i18n.t('answerFloatCloseClicks') }} <output>{{ settings.closeClicks }}</output></span>
          <input v-model.number="settings.closeClicks" type="range" min="2" max="5" step="1" />
        </label>
        <label class="float-setting">
          <span>{{ i18n.t('answerFloatCloseSeconds') }} <output>{{ settings.closeSeconds }} s</output></span>
          <input v-model.number="settings.closeSeconds" type="range" min="1" max="10" step="1" />
        </label>
        <label class="float-setting">
          <span>{{ i18n.t('answerFloatShortcut') }}</span>
          <input class="float-shortcut" :value="shortcutDraft" readonly @keydown="captureShortcut" :placeholder="i18n.t('answerFloatShortcutCapture')" />
        </label>
        <p class="mt-1">{{ i18n.t('answerFloatShortcutCapture') }}</p>
        <button class="float-setting-action" :disabled="shortcutBusy" @click="saveShortcut">{{ i18n.t('answerFloatShortcutSave') }}</button>
        <p v-if="shortcutError" role="alert" class="mt-1" style="color: rgb(var(--md-error))">{{ shortcutError }}</p>
        <p v-if="shortcutReady" class="mt-1">{{ i18n.t('answerFloatShortcutActive') }}: {{ settings.shortcut }}</p>
        <p class="mt-2">{{ i18n.t(isTopmost ? 'answerFloatLayerTop' : 'answerFloatLayerBack') }}</p>
        <button class="float-setting-action" :disabled="layerBusy || (!shortcutReady && isTopmost)" @click="setLayer(!isTopmost)">
          {{ i18n.t(isTopmost ? 'answerFloatSendBack' : 'answerFloatBringTop') }}
        </button>
      </div>
      <div v-if="closeAttempts || closeBlocked" class="float-settings shrink-0" role="status" aria-live="polite">
        {{ closeAttempts
          ? i18n.t('answerFloatCloseProgress', { remaining: settings.closeClicks - closeAttempts, seconds: settings.closeSeconds })
          : i18n.t('answerFloatCloseInstruction', { count: settings.closeClicks, seconds: settings.closeSeconds }) }}
      </div>
      <button v-if="topmostError" class="float-settings shrink-0 text-left" @click="restoreTopmost">
        {{ i18n.t('answerFloatTopmostRetry') }}
      </button>

      <div v-if="adjusting && hasBegun" class="shrink-0 px-3.5 pb-1.5">
        <div class="float-paused">
          <PauseCircleIcon class="w-3.5 h-3.5 shrink-0" />
          <span class="truncate">{{ i18n.t('searchScreenRecordPaused') }}</span>
        </div>
      </div>

      <div class="float-body flex-1 overflow-y-auto px-3.5 pb-3 min-h-0">
        <div v-if="adjusting" class="flex flex-col items-center justify-center h-full gap-3">
          <button
            class="float-begin-btn"
            @mousedown.stop
            @click="handleBegin"
          >
            <PlayIcon class="w-6 h-6" />
          </button>
          <div class="text-center">
            <p class="text-[15px] font-semibold" style="color: var(--accent);">
              {{ hasBegun ? i18n.t('searchScreenRecordResume') : i18n.t('searchScreenRecordStart') }}
            </p>
            <p v-if="!hasBegun" class="text-[11px] mt-1" style="color: var(--fg2);">
              先调整录制框，再点击开始
            </p>
          </div>
        </div>

        <div v-else-if="store.currentResult" :key="store.currentResult.question.id" class="space-y-2 pt-0.5">
          <div class="float-answer">
            <CheckIcon class="w-4 h-4 shrink-0" />
            <span class="text-[13px] font-bold">
              {{ i18n.t('searchScreenRecordAnswer') }}: {{ store.currentResult.question.answer }}
            </span>
          </div>

          <p class="text-[13px] leading-snug" style="color: var(--fg);">
            {{ store.currentResult.question.stem }}
          </p>

          <div v-if="store.currentResult.question.options?.length" class="space-y-1">
            <div
              v-for="(opt, idx) in store.currentResult.question.options"
              :key="idx"
              class="float-option"
              :class="{ 'float-option-correct': isCorrect(idx) }"
            >
              <div class="float-letter" :class="{ 'float-letter-correct': isCorrect(idx) }">
                <span>{{ String.fromCharCode(65 + idx) }}</span>
              </div>
              <span class="float-option-text">{{ opt }}</span>
            </div>
          </div>

          <p class="float-bank pt-0.5" style="color: var(--fg2);">
            {{ store.currentResult.bankName }}
          </p>
        </div>

        <div v-else-if="store.ocrError" class="flex flex-col items-center justify-center h-full gap-2.5">
          <div class="float-empty-icon" style="color: rgb(var(--md-error));">
            <XMarkIcon class="w-5 h-5" />
          </div>
          <p class="text-[13px] text-center" style="color: rgb(var(--md-error));">
            OCR 初始化失败
          </p>
          <p class="text-[11px] text-center px-2" style="color: var(--fg2);">
            {{ store.ocrError }}
          </p>
        </div>

        <div v-else class="flex flex-col items-center justify-center h-full gap-2.5">
          <div class="float-empty-icon">
            <MagnifyingGlassIcon class="w-5 h-5" />
          </div>
          <p class="text-[13px]" style="color: var(--fg2);">
            {{ i18n.t('searchScreenRecordNoMatch') }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
html, body, #app {
  width: 100%;
  height: 100%;
  background: transparent !important;
  margin: 0;
  padding: 0;
  overflow: hidden;
}
</style>

<style scoped>
.float-card {
  --fg: rgb(var(--md-on-surface));
  --fg2: rgb(var(--md-on-surface-variant));
  --fill: rgb(var(--md-surface-container-high) / var(--background-opacity));
  --accent: rgb(var(--md-primary));
  background: rgb(var(--md-surface-container) / var(--background-opacity));
  border-radius: 16px;
  border: 1px solid rgb(var(--md-outline-variant) / var(--background-opacity));
  overflow: hidden;
}

.float-grip {
  width: 36px;
  height: 5px;
  border-radius: 999px;
  background: rgb(var(--md-outline-variant) / var(--background-opacity));
}

.float-actions {
  --button-background-opacity: var(--background-opacity);
  --button-icon-opacity: var(--text-opacity);
}
/* Keep transparent controls recoverable without changing the saved settings. */
.float-actions:hover {
  --button-background-opacity: 1;
  --button-icon-opacity: 1;
}
.float-btn {
  width: 28px;
  height: 28px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  cursor: pointer;
  color: var(--fg);
  background: rgb(var(--md-surface-container-high) / var(--button-background-opacity));
  transition: filter 0.15s ease, transform 0.1s ease;
}
.float-btn > svg {
  opacity: var(--button-icon-opacity);
}
.float-btn:focus-visible {
  --button-background-opacity: 1;
  --button-icon-opacity: 1;
  outline: 2px solid rgb(var(--md-primary));
  outline-offset: 2px;
}
.float-btn:hover {
  filter: brightness(0.94);
}
.float-btn:active {
  transform: scale(0.88);
}
.float-btn-danger {
  color: rgb(var(--md-error));
}

.float-paused {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  color: rgb(var(--md-on-tertiary-container));
  background: rgb(var(--md-tertiary-container) / var(--background-opacity));
}

.float-answer {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 999px;
  color: rgb(var(--md-on-primary-container));
  background: rgb(var(--md-primary-container) / var(--background-opacity));
  animation: float-pop 0.25s ease;
}

.float-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 12px;
  color: var(--fg2);
  background: var(--fill);
}
.float-option-correct {
  color: rgb(var(--md-on-primary-container));
  font-weight: 600;
  background: rgb(var(--md-primary-container) / var(--background-opacity));
}

.float-letter {
  width: 20px;
  height: 20px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 10px;
  font-weight: 700;
  color: var(--fg2);
  background: rgb(var(--md-surface-container-highest) / var(--background-opacity));
}
.float-letter-correct {
  color: rgb(var(--md-on-primary));
  background: rgb(var(--md-primary) / var(--background-opacity));
}

.float-empty-icon {
  width: 44px;
  height: 44px;
  border-radius: 999px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--fg2);
  background: var(--fill);
}

.float-title, .float-paused span, .float-body p, .float-body span {
  opacity: var(--text-opacity);
}
.float-body {
  font-size: var(--answer-font-size);
  overflow-wrap: anywhere;
}
.float-body p, .float-answer span, .float-option-text {
  font-size: inherit;
}
.float-body .float-bank { font-size: 0.85em; }
.float-answer { border-radius: 12px; max-width: 100%; }
.float-option { align-items: flex-start; }
.float-option-text { min-width: 0; white-space: normal; }
.float-letter { width: 1.7em; height: 1.7em; font-size: 0.8em; }
.float-settings {
  color: rgb(var(--md-on-surface));
  background: rgb(var(--md-surface-container-high));
  padding: 10px 14px;
  font-size: 12px;
  max-height: 55%;
  overflow-y: auto;
}
.float-setting { display: block; margin-top: 5px; }
.float-setting span { display: flex; justify-content: space-between; gap: 8px; }
.float-setting input[type="range"] { display: block; width: 100%; accent-color: rgb(var(--md-primary)); }

.float-shortcut {
  width: 100%; margin-top: 6px; padding: 6px;
  border: 1px solid rgb(var(--md-outline)); border-radius: 6px;
  color: rgb(var(--md-on-surface)); background: rgb(var(--md-surface));
}
.float-setting-action { margin-top: 6px; padding: 5px 9px; border-radius: 6px; background: rgb(var(--md-primary-container)); color: rgb(var(--md-on-primary-container)); }
.float-setting-action:disabled { opacity: 0.5; cursor: not-allowed; }
.float-body::-webkit-scrollbar {
  width: 4px;
}
.float-body::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: var(--fill);
}

@keyframes float-pop {
  from { transform: scale(0.85); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.float-begin-btn {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  color: rgb(var(--md-on-primary));
  background: rgb(var(--md-primary));
  cursor: pointer;
  box-shadow: 0 6px 20px rgba(var(--md-primary) / 0.35);
  transition: transform 0.15s ease, filter 0.15s ease;
}
.float-begin-btn:hover {
  filter: brightness(1.06);
  transform: scale(1.04);
}
.float-begin-btn:active {
  transform: scale(0.96);
}
</style>
