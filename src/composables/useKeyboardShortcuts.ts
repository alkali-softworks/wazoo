import { onMounted, onBeforeUnmount } from 'vue'
import type { Ref } from 'vue'
import { usePlayerStore } from '@/stores/player'
import { useSettingsStore } from '@/stores/settings'
//import { log } from '@/lib/utils'

export function useKeyboardShortcuts(options: {
  showSearchModal: Ref<boolean>,
  showMenuModal: Ref<boolean>,
  showSettingsModal: Ref<boolean>,
  showHelpModal: Ref<boolean>,
  showWazooControls: Ref<boolean>,
  getScrollSpeed: () => void,
  setScrollSpeed: (n: number) => void,
  setNPlayers: (n: number) => void,
  toggleLayout: () => void,
  toggleScrollMode: () => void,
  showNotice: (msg: string) => void,
  addNewPlayer: () => void,
  globalUnmute: () => void,
  toggleGlobalMute: () => void,
  isScrollMode: Ref<boolean>,
}) {
  const playerStore = usePlayerStore()
  const settingsStore = useSettingsStore()

  function handleKeyDown(e: KeyboardEvent) {

    // Return early if modal is open
    if (options.showSearchModal.value) return
    if (options.showMenuModal.value) return
    if (options.showSettingsModal.value) return
    if (options.showHelpModal.value) return

    //log('Wazoo handleKeyDown:', e.key)

    if (e.key === 'ArrowUp' && e.type === 'keydown') {
      e.preventDefault()
      playerStore.playNextVideo(playerStore.currentPlayer.id)
      return
    }

    if (e.key === 'ArrowDown' && e.type === 'keydown') {
      e.preventDefault()
      playerStore.playPrevious(playerStore.currentPlayer.id)
      return
    }

    if (e.key === 'ArrowLeft' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.seek(-5)
      return
    }

    if (e.key === 'ArrowRight' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.seek(5)
      return
    }

    if (e.key === ',' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.previousFrame()
      return
    }

    if (e.key === '.' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.nextFrame()
      return
    }

    if (e.key === 'x' && e.type === 'keydown') {
      e.preventDefault()
      if (playerStore.currentPlayer) {
        playerStore.removePlayer(playerStore.currentPlayer.id)
        settingsStore.setPlayerCount(playerStore.players.length)
      }
      return
    }

    if (e.key === 'n' && e.type === 'keydown') {
      e.preventDefault()
      options.addNewPlayer()
      return
    }

    if (e.key === 'l' && e.type === 'keydown') {
      e.preventDefault()
      options.toggleLayout()
      return
    }

    if (e.key === '1' && e.type === 'keydown') {
      e.preventDefault()
      options.setNPlayers(1)
      return
    }

    if (e.key === '2' && e.type === 'keydown') {
      e.preventDefault()
      options.setNPlayers(2)
      return
    }

    if (e.key === '3' && e.type === 'keydown') {
      e.preventDefault()
      options.setNPlayers(3)
      return
    }

    if (e.key === '4' && e.type === 'keydown') {
      e.preventDefault()
      options.setNPlayers(4)
      return
    }

    if (e.key === '5' && e.type === 'keydown') {
      e.preventDefault()
      options.toggleScrollMode()
      return
    }

    if (e.key === 's' && e.type === 'keydown') {
      e.preventDefault()
      if (playerStore.currentPlayer) {
        playerStore.togglePlayMode(playerStore.currentPlayer.id)
        options.showNotice(`Switched to ${playerStore.currentPlayer.playMode} mode`)
      }
      return
    }

    if (e.key === 'Tab' && e.type === 'keydown') {
      e.preventDefault()
      playerStore.setNextPlayerFocused()
      return
    }

    if (e.key === 'c' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.toggleSubtitles()
      return
    }

    if ((e.key === 'j' || e.key === '/') && e.type === 'keydown') {
      e.preventDefault()
      options.showSearchModal.value = true
      return
    }

    if (e.key === '?' && e.type === 'keydown') {
      options.showHelpModal.value = true
      return
    }

    if (e.key === 'Escape' && e.type === 'keydown') {
      options.showMenuModal.value = true
      return
    }

    if (e.key === ']' && e.type === 'keydown') {
      e.preventDefault()
      if (options.isScrollMode.value) {
        options.globalUnmute()
        return
      }
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.adjustVolume(0.1)
      options.showNotice(focusedPlayer.getVolume())
      return
    }

    if (e.key === '[' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.adjustVolume(-0.1)
      options.showNotice(focusedPlayer.getVolume())
      return
    }

    if (e.key === '-' && e.type === 'keydown') {
      e.preventDefault()
      let newSpeed = Number(options.getScrollSpeed()) - 0.1
      options.setScrollSpeed(newSpeed)
      options.showNotice('Scroll Speed Down: ' + Math.round(newSpeed * 100) / 100)
      return
    }
    if ((e.key === '+' || e.key === '=') && e.type === 'keydown') {
      e.preventDefault()
      let newSpeed = Number(options.getScrollSpeed()) + 0.1
      options.setScrollSpeed(newSpeed)
      options.showNotice('Scroll Speed Up: ' + Math.round(newSpeed * 100) / 100)
      return
    }

    if (e.key === 'm' && e.type === 'keydown') {
      e.preventDefault()
      if (options.isScrollMode.value) {
        options.toggleGlobalMute()
        return
      }
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.toggleMute()
      return
    }

    if (e.key === 'h' && e.type === 'keydown') {
      e.preventDefault()
      options.showWazooControls.value = !options.showWazooControls.value
      return
    }

    if (e.key === ' ' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.togglePlay()
      return
    }

    if ((e.key === 'f' || e.key === 'F11') && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.toggleFullscreen()
      return
    }

    if (e.key === 't' && e.type === 'keydown') {
      e.preventDefault()
      const focusedPlayer = playerStore.getFocusedPlayer()
      if (!focusedPlayer) return
      focusedPlayer.showOverlay()
      return
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown)
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', handleKeyDown)
  })

  return {
    handleKeyDown
  }
}