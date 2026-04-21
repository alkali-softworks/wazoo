import App from './App.vue'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createI18n } from 'vue-i18n'
import './styles/globals.css'

import en from './locales/en.json'
import es from './locales/es.json'
import ja from './locales/ja.json'
import zh from './locales/zh.json'
import fr from './locales/fr.json'
import de from './locales/de.json'
import ru from './locales/ru.json'
import it from './locales/it.json'
import pt from './locales/pt.json'
import ko from './locales/ko.json'
import hi from './locales/hi.json'
import bn from './locales/bn.json'
import ar from './locales/ar.json'
import id from './locales/id.json'
import he from './locales/he.json'
import th from './locales/th.json'

const i18n = createI18n({
  legacy: false,
  locale: 'en',
  fallbackLocale: 'en',
  messages: {
    en,
    es,
    ja,
    zh,
    fr,
    de,
    ru,
    it,
    pt,
    ko,
    hi,
    bn,
    ar,
    id,
    he,
    th
  }
})

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(i18n)

app.mount('#app')
