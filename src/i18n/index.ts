import { createI18n } from 'vue-i18n'

import en from './messages/en'
import ja from './messages/ja'
import ko from './messages/ko'
import zh from './messages/zh'

export const i18n = createI18n({
  legacy: false,
  locale: 'zh',
  fallbackLocale: 'en',
  messages: {
    zh,
    en,
    ja,
    ko,
  },
})
