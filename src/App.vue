<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useI18n } from 'vue-i18n'
import Wazoo from '@/components/Wazoo.vue'
import { log } from '@/lib/utils'

const { t, locale } = useI18n()
const isElectron = window.electron !== undefined
const settingsStore = useSettingsStore()
const wazooRef = ref<InstanceType<typeof Wazoo>>()
const appicon = './icon.png'
let timeoutId: any = null

const showTitlebar = ref(false)
const isMenuOpen = ref(false)
const isAltPressed = ref(false)

// Watch for language changes in settings store
watch(() => settingsStore.language, (newLang) => {
  if (newLang) {
    locale.value = newLang
    // Handle RTL for Arabic and Hebrew
    if (newLang === 'ar' || newLang === 'he') {
      document.documentElement.setAttribute('dir', 'rtl')
    } else {
      document.documentElement.setAttribute('dir', 'ltr')
    }
  }
}, { immediate: true })

const closeApp = () => {
  window.electron.invoke('close-app')
}

const toggleMaximize = () => {
  window.electron.invoke('toggle-maximize-window')
}

const minimize = () => {
  window.electron.invoke('minimize-window')
}

const toggleMenu = () => {
  isMenuOpen.value = !isMenuOpen.value
}

const handleMouseMove = (event: MouseEvent) => {
  if (event.clientY < 35 || isMenuOpen.value) {
    showTitlebar.value = true
    if (timeoutId) clearTimeout(timeoutId)
  } else {
    if (timeoutId) clearTimeout(timeoutId)
    timeoutId = setTimeout(() => {
      if (!isMenuOpen.value) {
        showTitlebar.value = false
      }
    }, 400)
  }
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Alt') {
    isAltPressed.value = true
  }
  if (isAltPressed.value && e.key === 'x') {
    window.electron.invoke('close-app');
  }
}

const handleKeyUp = (e: KeyboardEvent) => {
  //log('handleKeyUp: ',e.key)
  if (e.key === 'Alt') {
    isAltPressed.value = false
  }
}

const releaseAltKey = () => {
  isAltPressed.value = false
}

const handleMenuClick = (action: string) => {
  isMenuOpen.value = false
  switch(action) {
    case 'search':
      wazooRef.value.openSearchModal()
      break;
    case 'add-player':
      wazooRef.value.addNewPlayer()
      break;
    case 'toggle-layout':
      wazooRef.value.toggleLayout()
      break;
    case 'toggle-files':
      wazooRef.value.toggleFiles()
      break;
    case 'settings':
      wazooRef.value.openSettingsModal()
      break;
    case 'help':
      wazooRef.value.openHelpModal()
      break;
    case 'quit':
      window.electron.invoke('close-app')
      break;
  }
}

watch(isAltPressed, (isPressed) => {
  console.log('isAltPressed', settingsStore.windowOpacity)
  if (isPressed) {
    window.electron.invoke('set-window-opacity', 0.85)
  } else {
    window.electron.invoke('set-window-opacity', settingsStore.windowOpacity)
  }
})

onMounted(() => {
  if (isElectron) {
    window.addEventListener('mouseup', releaseAltKey)
    window.addEventListener('mouseleave', releaseAltKey)
    window.addEventListener('mousemove', handleMouseMove)

    window.addEventListener('keydown', handleKeyDown)
    window.addEventListener('keyup', handleKeyUp)

    window.addEventListener('blur', releaseAltKey)
    window.addEventListener('focus', releaseAltKey)
    window.addEventListener('resize', releaseAltKey)

    // Close menu when clicking outside
    window.addEventListener('click', (e) => {
      if (!(e.target as HTMLElement).closest('.titlebar-content')) {
        isMenuOpen.value = false
      }
    })
  }
})

onUnmounted(() => {
  if (isElectron) {
    window.removeEventListener('mouseup', releaseAltKey)
    window.removeEventListener('mouseleave', releaseAltKey)
    window.removeEventListener('mousemove', handleMouseMove)

    window.removeEventListener('keydown', handleKeyDown)
    window.removeEventListener('keyup', handleKeyUp)

    window.removeEventListener('blur', releaseAltKey)
    window.removeEventListener('focus', releaseAltKey)
    window.removeEventListener('resize', releaseAltKey)
    if (timeoutId) clearTimeout(timeoutId)
  }
})

</script>

<template>

  <template v-if="isElectron">
    <div v-if="isAltPressed" class="drag-overlay">{{ t('app.drag_to_move') }}<br/>{{ t('app.x_to_quit') }}</div>
    <div class="titlebar" :class="{ 'titlebar-visible': showTitlebar }">
      <div class="titlebar-content" @click="toggleMenu">
        <img :src="appicon" class="titlebar-icon" alt="icon" />
        Wazoo
        <div v-if="isMenuOpen" class="menu">
          <div class="menu-item" @click="handleMenuClick('add-player')">{{ t('common.add_player') }}</div>
          <div class="menu-item" @click="handleMenuClick('toggle-layout')">{{ t('common.toggle_layout') }}</div>
          <div class="menu-item" @click="handleMenuClick('toggle-files')">{{ t('common.toggle_files') }}</div>
          <div class="menu-item" @click="handleMenuClick('search')">{{ t('common.search') }}</div>
          <div class="menu-item" @click="handleMenuClick('settings')">{{ t('common.settings') }}</div>
          <div class="menu-item" @click="handleMenuClick('help')">{{ t('common.help') }}</div>
          <div class="menu-item" @click="handleMenuClick('quit')">{{ t('common.quit') }}</div>
        </div>
      </div>
      <div class="window-controls">
        <div class="minimize-button" @click.stop.prevent="minimize">─</div>
        <div class="maximize-button" @click.stop.prevent="toggleMaximize">□</div>
        <div class="close-button" @click.stop.prevent="closeApp">✕</div>
      </div>
    </div>
  </template>

  <Suspense>
    <template #default>
      <Wazoo ref="wazooRef" />
    </template>
    <template #fallback>
      {{ t('common.loading') }}
    </template>
  </Suspense>

</template>

<style>

html, body {
  margin: 0;
  padding: 0;
}
::-webkit-scrollbar {
  display: none;
}

.drag-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 99999;
  -webkit-app-region: drag;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  text-align: center;
  align-items: center;
  justify-content: center;
  font-weight: bold;
  color: white;
}

.titlebar-wrapper {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 35px;
  z-index: 9999;
}

.titlebar {
  height: 30px;
  -webkit-app-region: drag;
  background: #1f092f;
  width: 100%;
  position: fixed;
  top: -30px;
  left: 0;
  transition: top 0.1s ease;
  display: flex;
  justify-content: space-between;
  z-index: 9999;
}

.titlebar-visible {
  top: 0;
}

.titlebar-content {
  -webkit-app-region: no-drag;
  padding-left: 6px;
  padding-right: 12px;
  padding-top: 2px;
  font-size: 17px;
  font-weight: 600;
  height: 100%;
  background-color: #000000;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border-top-right-radius: 16px;
  border-bottom-right-radius: 16px;
  position: relative;
  user-select: none;
  -webkit-user-select: none;
}

.titlebar-icon {
  width: 24px;
  height: 24px;
}

.window-controls {
  display: flex;
  justify-content: flex-end;
}

.menu {
  position: absolute;
  top: 100%;
  left: 0;
  background: #000000;
  border: 1px solid #333;
  border-top: none;
  border-radius: 0 0 6px 6px;
  z-index: 10000;
}

.menu-item {
  padding: 8px 12px;
  cursor: pointer;
  color: #fff;
  transition: background-color 0.2s;
  white-space: nowrap;
}

.menu-item:hover {
  background: #1f1f1f;
}

.menu-item:last-child {
  border-radius: 0 0 6px 6px;
}

.close-button {
  -webkit-app-region: no-drag;
  height: 100%;
  width: 46px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 16px;
  color: #888;
  transition: background-color 0.2s, color 0.2s;
  z-index: 10000;
  position: relative;
}

.close-button:hover {
  background-color: #e81123;
  color: white;
}

.maximize-button {
  -webkit-app-region: no-drag;
  height: 100%;
  width: 46px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 16px;
  color: #888;
  transition: background-color 0.2s, color 0.2s;
  z-index: 10000;
  position: relative;
}

.maximize-button:hover {
  background-color: #333;
  color: white;
}

.minimize-button {
  -webkit-app-region: no-drag;
  height: 100%;
  width: 46px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 16px;
  color: #888;
  transition: background-color 0.2s, color 0.2s;
  z-index: 10000;
  position: relative;
}

.minimize-button:hover {
  background-color: #333;
  color: white;
}

</style>