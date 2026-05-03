import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { i18n } from '../i18n'
import type { AppSettings, LogEntry } from '../types'

interface BootstrapPayload {
  settings: AppSettings
  config_dir: string
}

function resolveSystemLocale() {
  const language = navigator.language.toLowerCase()
  if (language.startsWith('ja')) {
    return 'ja'
  }
  if (language.startsWith('ko')) {
    return 'ko'
  }
  if (language.startsWith('zh')) {
    return 'zh'
  }
  return 'en'
}

export const useAppStore = defineStore('app', {
  state: () => ({
    ready: false,
    configDir: '',
    settings: {
      language: 'system',
      launch_animation: true,
      particle_level: 'high',
      window_width: 1280,
      window_height: 800,
      log_to_disk: false,
    } as AppSettings,
    logs: [] as LogEntry[],
    logPanelOpen: false,
  }),
  getters: {
    recentLogs: (state) => state.logs.slice(-300),
    currentLocale: (state) =>
      state.settings.language === 'system' ? resolveSystemLocale() : state.settings.language,
  },
  actions: {
    async bootstrap() {
      const payload = await invoke<BootstrapPayload>('bootstrap')
      this.settings = payload.settings
      this.configDir = payload.config_dir
      this.applyLanguage()
      this.ready = true
    },
    async initLogListener() {
      await listen<LogEntry>('app://log', (event) => {
        this.logs.push(event.payload)
        if (this.logs.length > 5000) {
          this.logs.shift()
        }
      })
    },
    async saveSettings(next: AppSettings) {
      await invoke('save_settings', { settings: next })
      this.settings = next
      this.applyLanguage()
    },
    applyLanguage() {
      i18n.global.locale.value = this.currentLocale as 'zh' | 'en' | 'ja' | 'ko'
    },
    addLocalLog(level: string, source: string, message: string) {
      this.logs.push({
        level,
        source,
        message,
        timestamp: new Date().toISOString(),
      })
    },
  },
})
