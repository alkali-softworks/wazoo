<script setup lang="ts">
import { IVideoResult } from '@/types'
import { ref, onMounted, onBeforeUnmount, provide, nextTick, watch, toRaw } from 'vue'
import { usePlayerStore } from '@/stores/player'
import { useSettingsStore } from '@/stores/settings'
import { useAppState } from '@/stores/app'
import { useI18n } from 'vue-i18n'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import Player from '@/components/Player.vue'
//import bobClient from '@/lib/bobClient'
import MenuModal from '@/components/MenuModal.vue'
import SettingsModal from '@/components/SettingsModal.vue'
import SearchModal from '@/components/SearchModal.vue'
import HelpModal from '@/components/HelpModal.vue'
import Notice from '@/components/Notice.vue'
import FilePicker from '@/components/FilePicker.vue'
import { log, debounce } from '@/lib/utils'
import { FileQuestion, FolderOpen, Loader2 } from 'lucide-vue-next'


const settingsStore = useSettingsStore()
await settingsStore.loadSettings()
const playerStore = usePlayerStore()
const appState = useAppState()
const { t } = useI18n()

// Non-reactive state
const state = {
  searchQuery: '',
  searchFolder: 'All' as string | string[],
  isGlobalMuted: true,
  scrollSpeed: 1,
  lastTotalVideos: 0,
  totalVideos: 0,
  scrollInterval: null as number | null,
  playerPositions: new Map<number, number>(), // <playerId, yPos>
  playerHeights: new Map<number, number>() // <playerId, height>
}

// Reactive refs for template bindings
const isRowLayout = ref(false)
const isGridLayout = ref(false)
const isScrollMode = ref(false)
const showWazooControls = ref(true)
const showMenuModal = ref(false)
const showSettingsModal = ref(false)
const showSearchModal = ref(false)
const showHelpModal = ref(false)
const showNoResults = ref(false)
const showSetup = ref(false)
const noticeRef = ref()
const searchModalRef = ref()

const showScanToast = ref(false)
const scanToastMessage = ref('')
let scanToastTimeout: ReturnType<typeof setTimeout> | null = null

watch(() => appState.lastVideoScanned, (newVal) => {
  console.log('lastVideoScanned', newVal)
  if (newVal) {
    scanToastMessage.value = newVal
    showScanToast.value = true
    if (scanToastTimeout) clearTimeout(scanToastTimeout)
    scanToastTimeout = setTimeout(() => {
      showScanToast.value = false
    }, 2000)
  }
})

async function checkSetupNeeded() {
  if (!settingsStore.mediaFolders || settingsStore.mediaFolders.length === 0) {
    showSetup.value = true
    return true
  }
  const result = await window.electron.invoke('get-video-count')
  if (result.success && result.count === 0) {
    showSetup.value = true
    return true
  }
  showSetup.value = false
  return false
}

function addNewPlayer() {
  playerStore.addPlayer()
  settingsStore.setPlayerCount(playerStore.players.length)
}

async function initFiles() {
  if (settingsStore.lastQuery != '') {
    state.searchQuery = settingsStore.lastQuery
    state.searchFolder = settingsStore.lastFolder

    searchModalRef.value.setQuery(state.searchQuery)
    searchModalRef.value.setFolder(state.searchFolder)
  }
  await queryFiles(state.searchQuery, state.searchFolder)
}

async function handleSearch(query: string, folder: string | string[]) {
  // Save current state before changing query
  playerStore.saveQueryState()
  state.searchQuery = query
  state.searchFolder = folder
  appState.setInitialLoad(true)
  queryFiles(query, folder)
  settingsStore.setLastQuery(query, folder)
}

// async function queryFiles(query: string, folder: string) {
//   const res = await bobClient.get<IVideoList>('?q=' + query + '&folder=' + folder)
//   const videos = res.map((item) => bobClient.baseUrl() + item[0])
//   const codecs = res.map((item) => item[1])
//   videoTotals(videos.length, folder)
//   playerStore.clearVideos()
//   playerStore.addVideos(videos)
//   playerStore.addCodecs(codecs)
// }

async function queryFiles(query: string, folder: string | string[]) {
  appState.setInitialLoad(true)
  appState.setQuery(query)
  const result = await window.electron.invoke('search-videos', query, toRaw(folder)) as IVideoResult
  if (result.success) {
    const videos = result.videos.map(v => `file://${v.path}`)
    const codecs = result.videos.map(v => v.codec)
    await videoTotals(videos.length, folder)
    playerStore.clearVideos()
    playerStore.addVideos(videos, codecs)
  } else {
    showNotice(t('wazoo.search_error', { error: result.error }))
  }
  appState.setInitialLoad(false)
}

async function videoTotals(videosLength: number, folder: string | string[]){
  state.totalVideos = videosLength

  // Check for zero results
  if (state.totalVideos === 0) {
    playerStore.clearVideos()
    const setupNeeded = await checkSetupNeeded()
    if (setupNeeded) {
      showNoResults.value = false
      return
    } else {
      showNoResults.value = true
      let folderLabel = 'All'
      if (Array.isArray(folder)) {
        if (folder.length === 0 || folder.includes('All')) {
          folderLabel = 'All'
        } else {
          folderLabel = folder.length > 2 
            ? t('search.multiple_folders', { count: folder.length })
            : folder.map(f => f.split(/[/\\]/).filter(Boolean).pop() || f).join(', ')
        }
      } else {
        folderLabel = folder
      }
      showNotice(t('wazoo.no_files_found_in', { folder: folderLabel }))
      return
    }
  } else {
    // We have videos
    showNoResults.value = false
    showSetup.value = false
  }

  if (state.lastTotalVideos != 0 && state.totalVideos > state.lastTotalVideos) {
    const diff = (state.totalVideos - state.lastTotalVideos).toLocaleString() 
    showNotice(`${t('wazoo.total_files', { total: state.totalVideos.toLocaleString() })}<br>${t('wazoo.more_than_before', { diff })}`)
  }
  else if (state.lastTotalVideos != 0 && state.totalVideos < state.lastTotalVideos) {
    const diff = (state.lastTotalVideos - state.totalVideos).toLocaleString()
    showNotice(`${t('wazoo.total_files', { total: state.totalVideos.toLocaleString() })}<br>${t('wazoo.less_than_before', { diff })}`)
  } else {
    showNotice(t('wazoo.total_files', { total: state.totalVideos.toLocaleString() }))
  }
  state.lastTotalVideos = state.totalVideos
}

function handleVideoEvent(evt: {
  event: Event
  playerId: number
  videoRef: InstanceType<typeof HTMLVideoElement>
}) {
  // Listen for mute toggles to update global state
  if (evt.event.type === 'volumechange') {
    checkGlobalMuteState()
  }

  if (evt.event.type === 'seeked') {
    if (!appState.initialLoad) {
      playerStore.saveQueryState()
    }
  }

  if (evt.event.type === 'loadedmetadata') {
    playerStore.loadQueryState()

    if (!appState.initialLoad) {
      //log('loadedmetadata calling Saving query state')
      playerStore.saveQueryState()
    }
    if (isScrollMode.value) {
      // Seek to random position
      if (evt.videoRef && isFinite(evt.videoRef.duration)) {
        const randomTime = Math.random() * evt.videoRef.duration
        evt.videoRef.currentTime = randomTime
      }

      const playerRef = playerStore.getPlayerRef(evt.playerId)
      if (playerRef) {
        const height = playerRef.getHeight()
        if (height > 0) {
          const oldHeight = state.playerHeights.get(evt.playerId)
          // Only update if height is known and has changed significantly
          if (oldHeight === undefined || oldHeight === 0 || Math.abs(oldHeight - height) > 1) {
            log(`Player ${evt.playerId} height set to: ${height}`)
            state.playerHeights.set(evt.playerId, height)
            recalculateScrollPositions() 
          }
        }
      }
    }
    return
  }

  if (evt.event.type === 'ended') {
    playerStore.playNextVideo(evt.playerId)
    return
  }

  if (evt.event.type === 'playing') {
    const playerRef = playerStore.getPlayerRef(evt.playerId)
    playerRef.setPlaying(true)
    //showNotice(formatVideoTitle(playerStore.currentPlayer.src))
    return
  }

  if (evt.event.type === 'error') {
    if (evt.videoRef) {
      const error = evt.videoRef.error
      if (error) {
        log('Video Error:', {
          code: error.code,
          message: error.message
        })
      }
    }

    setTimeout(() => {
      playerStore.playNextVideo(evt.playerId)
    }, 500)

    return
  }
}

function globalUnmute() {
  state.isGlobalMuted = false
  // Force all currently active players to recognize they are unmuted
  playerStore.players.forEach(p => {
    const ref = playerStore.getPlayerRef(p.id)
    if (ref) {
      ref.setInternalMute(false)
      // Bump volume slightly so they don't stay silent until next scroll update
      ref.adjustVolume(0) 
    }
  })
  showNotice(t('wazoo.scroll_mode_unmuted'))
}

function toggleGlobalMute() {
  state.isGlobalMuted = !state.isGlobalMuted
  playerStore.players.forEach(p => {
    const ref = playerStore.getPlayerRef(p.id)
    if (ref) {
      ref.setInternalMute(state.isGlobalMuted)
    }
  })
  showNotice(state.isGlobalMuted ? t('wazoo.global_mode_muted') : t('wazoo.global_mode_unmuted'))
}

function checkGlobalMuteState() {
  if (!isScrollMode.value) return

  // Check if ANY player is manually unmuted
  let anyUnmuted = false
  for (const p of playerStore.players) {
    const pRef = playerStore.getPlayerRef(p.id)
    // We check user preference (getMuted), not just video.muted
    if (pRef && !pRef.getMuted()) {
      anyUnmuted = true
      break
    }
  }

  // If at least one is unmuted, Global Mute is OFF.
  // If all are muted, Global Mute is ON.
  state.isGlobalMuted = !anyUnmuted
}

function getScrollSpeed() {
  return state.scrollSpeed
}

function setScrollSpeed(n: number) {
  state.scrollSpeed = Math.min(Math.max(n, 0.1), 10);
}

function setNPlayers(n: number) {
  const targetCount = Math.max(1, Math.min(n, 12)) 
  const currentCount = playerStore.players.length
  if (targetCount === currentCount) return
  if (targetCount < currentCount) {
    while (playerStore.players.length > targetCount) {
      const lastPlayer = playerStore.players[playerStore.players.length - 1]
      playerStore.removePlayer(lastPlayer.id)
    }
  } else {
    while (playerStore.players.length < targetCount) {
      playerStore.addPlayer()
    }
  }
  settingsStore.setPlayerCount(targetCount)
  showNotice(t('wazoo.set_players_count', { count: targetCount, suffix: targetCount !== 1 ? 's' : '' }))
}

async function toggleFiles() {
  showWazooControls.value = !showWazooControls.value
}

async function toggleLayout() {
  // Cycle through: column -> row -> grid
  if (!isRowLayout.value && !isGridLayout.value) {
    flipWinowDimensions()
    isRowLayout.value = true
    isGridLayout.value = false
    settingsStore.setLayout('row')
    showNotice(t('wazoo.layout_row'))
  } else if (isRowLayout.value) {
    isRowLayout.value = false
    isGridLayout.value = true
    settingsStore.setLayout('grid')
    showNotice(t('wazoo.layout_grid'))
  } else {
    flipWinowDimensions()
    isRowLayout.value = false
    isGridLayout.value = false
    settingsStore.setLayout('column')
    showNotice(t('wazoo.layout_column'))
  }
}

async function targetSquareWindow() {
  if (window.electron) {
    const bounds = await window.electron.invoke('get-window-bounds')
    const aspectRatio = bounds.width / bounds.height

    // If window is very wide or very tall
    if (aspectRatio > 2.5 || aspectRatio < 0.4) {
      // Calculate new dimensions maintaining same area
      const area = bounds.width * bounds.height
      // Target a more square aspect ratio, like 4:3
      const targetAspectRatio = 4 / 3
      const newHeight = Math.sqrt(area / targetAspectRatio)
      const newWidth = newHeight * targetAspectRatio

      // Center the new window dimensions
      const x = bounds.x + (bounds.width - newWidth) / 2
      const y = bounds.y + (bounds.height - newHeight) / 2

      // Resize window
      await window.electron.invoke('set-window-bounds', {
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(newWidth),
        height: Math.round(newHeight)
      })
    }
  }
}

async function flipWinowDimensions() {
  if (window.electron) {
    const bounds = await window.electron.invoke('get-window-bounds')
    const aspectRatio = bounds.width / bounds.height

    // If window is very wide or very tall
    if (aspectRatio > 2.5 || aspectRatio < 0.4) {
      // Flip dimensions - make height = old width, width = old height
      const newHeight = bounds.width
      const newWidth = bounds.height

      // Center the new window dimensions
      const x = bounds.x + (bounds.width - newWidth) / 2
      const y = bounds.y + (bounds.height - newHeight) / 2

      // Resize window
      await window.electron.invoke('set-window-bounds', {
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(newWidth),
        height: Math.round(newHeight)
      })
    }
  }
}

function openSearchModal() {
  showSearchModal.value = true
}

function openSettingsModal() {
  showSettingsModal.value = true
}

function openHelpModal() {
  showHelpModal.value = true
}

function showNotice(message: string) {
  noticeRef.value.show(message, 3000)
}

const debouncedRecalculate = debounce(() => {
  if (isScrollMode.value) {
    log('Window resized, recalculating scroll positions...')
    recalculateScrollPositions()
    checkPlayerFill() 
  }
}, 250) 

function handleResize() {
  if (isScrollMode.value) {
    debouncedRecalculate()
  }
}

// --- SCROLL FUNCTIONS ---

function getEffectiveHeight(playerId: number): number {
  const windowHeight = window.innerHeight
  const realHeight = state.playerHeights.get(playerId) || 0
  // Use real height if known (greater than 0), otherwise guess 50vh
  return realHeight > 0 ? realHeight : windowHeight / 2 
}

/**
 * Calculates visibility percentage and updates volume
 */
function updatePlayerVolumes() {
  if (!isScrollMode.value) return

  const windowHeight = window.innerHeight

  state.playerPositions.forEach((y, playerId) => {
    const pRef = playerStore.getPlayerRef(playerId)
    if (!pRef) return

    // ENFORCE GLOBAL MUTE
    if (state.isGlobalMuted) {
      pRef.setVolume(0)
      return
    }

    // Standard Cross-fade Logic
    const height = getEffectiveHeight(playerId)
    const videoTop = y
    const videoBottom = y + height
    const visibleTop = Math.max(0, videoTop)
    const visibleBottom = Math.min(windowHeight, videoBottom)
    const visibleHeight = Math.max(0, visibleBottom - visibleTop)
    
    // Calculate intersection ratio
    const percent = height > 0 ? visibleHeight / height : 0
    
    pRef.setVolume(percent)
  })
}

/**
 * Checks to add/remove players to fill the screen.
 */
async function checkPlayerFill() {
  if (!isScrollMode.value) return

  const windowHeight = window.innerHeight
  let bottomEdge = -Infinity
  let bottomPlayerId = -1

  // Find the bottom-most player's bottom edge using effective height
  state.playerPositions.forEach((y, id) => {
    const effectiveHeight = getEffectiveHeight(id)
    const playerBottom = y + effectiveHeight
    if (playerBottom > bottomEdge) {
      bottomEdge = playerBottom
      bottomPlayerId = id
    }
  })

  // If no players exist, set edge to bottom of screen to spawn one
  if (bottomPlayerId === -1) {
    bottomEdge = windowHeight
  }

  // --- SPAWN LOGIC ---
  // Use 'while' to fill the screen, but advance bottomEdge with guess
  while (bottomEdge < windowHeight) {
    const newPlayer = playerStore.addPlayer()

    // Ensure new players spawn matching the current global mute state
    newPlayer.initialMuted = state.isGlobalMuted

    const newY = bottomEdge
    const guessedHeight = windowHeight / 2 // Our fallback guess

    state.playerPositions.set(newPlayer.id, newY)
    // Store 0, not the guess. The guess is only for this function's loop logic.
    state.playerHeights.set(newPlayer.id, 0) 

    await nextTick() // Wait for Vue to render

    const newPlayerEl = document.querySelector(
      `[data-player-id="${newPlayer.id}"]`
    ) as HTMLElement
    if (newPlayerEl) {
      newPlayerEl.style.transform = `translateY(${newY}px)`
      newPlayerEl.style.zIndex = Math.floor(newY).toString()
    }
    
    // **Manually advance bottomEdge using the guess**
    // This prevents the loop from running infinitely
    bottomEdge += guessedHeight 
  }

  // --- DESPAWN LOGIC ---
  // Use effective height to despawn
  const windowTop = 0
  state.playerPositions.forEach((y, id) => {
    const effectiveHeight = getEffectiveHeight(id)
    
    if (y + effectiveHeight < windowTop) {
       // This player's bottom (known or guessed) is off-screen
      playerStore.removePlayer(id)
      state.playerPositions.delete(id)
      state.playerHeights.delete(id)
    }
  })
}

/**
 * Recalculates and applies all player positions gaplessly.
 */
function recalculateScrollPositions() {
  if (!isScrollMode.value) return

  const sortedPlayers = [...playerStore.players]
    .map((p) => ({ id: p.id, y: state.playerPositions.get(p.id) }))
    .filter((p) => p.y !== undefined) 
    .sort((a, b) => a.y! - b.y!)

  if (sortedPlayers.length === 0) {
    checkPlayerFill() 
    return
  }

  let currentTop = sortedPlayers[0].y!
  const windowHeight = window.innerHeight
  
  if (currentTop > 0) {
    currentTop = 0
  }
  
  // Check if total *effective* height is less than window height
  let totalEffectiveHeight = 0
  for (const player of sortedPlayers) {
    totalEffectiveHeight += getEffectiveHeight(player.id)
  }
  
  // If all players fit on screen, stack them from the *bottom* up
  if (totalEffectiveHeight < windowHeight) {
    currentTop = windowHeight - totalEffectiveHeight
  }

  for (const player of sortedPlayers) {
    const playerEl = document.querySelector(
      `[data-player-id="${player.id}"]`
    ) as HTMLElement
    if (!playerEl) continue

    // Stack using the effective height
    const effectiveHeight = getEffectiveHeight(player.id)

    state.playerPositions.set(player.id, currentTop)
    playerEl.style.transform = `translateY(${currentTop}px)`
    playerEl.style.zIndex = Math.floor(currentTop).toString()

    currentTop += effectiveHeight // Advance top by effective height
  }

  updatePlayerVolumes()
  checkPlayerFill()
}

/**
 * Main animation loop.
 */
function updateScroll() {
  const playerIds = Array.from(state.playerPositions.keys())

  for (const playerId of playerIds) {
    const playerEl = document.querySelector(
      `[data-player-id="${playerId}"]`
    ) as HTMLElement
    if (!playerEl) continue

    let currentPos = state.playerPositions.get(playerId)
    if (currentPos === undefined) continue

    currentPos -= state.scrollSpeed
    state.playerPositions.set(playerId, currentPos)

    playerEl.style.transform = `translateY(${currentPos}px)`
    playerEl.style.zIndex = Math.floor(currentPos).toString()
  }

  updatePlayerVolumes()
  checkPlayerFill()

  if (isScrollMode.value) {
    state.scrollInterval = requestAnimationFrame(updateScroll)
  }
}

/**
 * Toggles the scroll mode on and off.
 */
async function toggleScrollMode() {
  isScrollMode.value = !isScrollMode.value
  state.isGlobalMuted = true

  if (isScrollMode.value) {
    const windowHeight = window.innerHeight

    // Clear previous state
    state.playerPositions.clear()
    state.playerHeights.clear()

    // Reset any existing transforms
    playerStore.players.forEach((player) => {
      const playerEl = document.querySelector(
        `[data-player-id="${player.id}"]`
      ) as HTMLElement
      if (playerEl) {
        playerEl.style.transform = ''
      }
    })

    let cumulativeBottom = windowHeight
    const playersToPosition = [...playerStore.players].reverse()

    for (const player of playersToPosition) {
      const playerRef = playerStore.getPlayerRef(player.id)
      const playerHeight = playerRef ? playerRef.getHeight() : 0
      
      // **Use effective height for initial stacking**
      const effectiveHeight = playerHeight > 0 ? playerHeight : windowHeight / 2 

      const position = cumulativeBottom - effectiveHeight
      state.playerPositions.set(player.id, position)
      // Store the *real* height (0 if unknown)
      state.playerHeights.set(player.id, playerHeight) 

      cumulativeBottom = position 
    }

    await nextTick() 

    // Now apply initial transforms
    state.playerPositions.forEach((position, id) => {
      const playerEl = document.querySelector(`[data-player-id="${id}"]`) as HTMLElement
      if (playerEl) {
        playerEl.style.transform = `translateY(${position}px)`
        playerEl.style.zIndex = Math.floor(position).toString()
      }
    })

    // Start scroll animation
    state.scrollInterval = requestAnimationFrame(updateScroll)
    showNotice(t('wazoo.scroll_mode_enabled'))

    // Immediately check if we need to add more players
    checkPlayerFill()
  } else {
    // Stop scroll animation
    if (state.scrollInterval) {
      cancelAnimationFrame(state.scrollInterval)
      state.scrollInterval = null
    }
    // Reset positions and state
    state.playerPositions.clear()
    state.playerHeights.clear()
    const players = document.querySelectorAll('.player-container') as NodeListOf<HTMLElement>
    players.forEach((player) => {
      player.style.transform = ''
      player.style.zIndex = ''
    })
    showNotice(t('wazoo.scroll_mode_disabled'))
  }
}

// --- END OF SCROLL FUNCTIONS ---

useKeyboardShortcuts({
  showSearchModal,
  showMenuModal,
  showSettingsModal,
  showHelpModal,
  showWazooControls,
  getScrollSpeed,
  setScrollSpeed,
  setNPlayers,
  toggleLayout,
  toggleScrollMode,
  showNotice,
  addNewPlayer,
  globalUnmute,
  toggleGlobalMute,
  isScrollMode,
})

setInterval(() => {
  playerStore.players.forEach((player) => {
    const videoObject = player.ref?.getVideoObject()
    if (!videoObject) return
    //if (videoObject.paused) return

    // compare time with last time
    //log('UNSTUCK CHECKER:', player.ref?.getLastTime(), videoObject.currentTime)
    if (videoObject.currentTime == player.ref?.getLastTime()) {
      player.ref?.setLastTime(videoObject.currentTime)
      log('UNSTUCK CHECKER: PLAYING NEXT VIDEO')
      playerStore.playNextVideo(player.id)
      return
    }

    player.ref?.setLastTime(videoObject.currentTime)
  })
}, 50000)

onMounted(async () => { 
  await initFiles()
  await checkSetupNeeded()
  if (!settingsStore.playerCount || isNaN(settingsStore.playerCount)) {
    settingsStore.setPlayerCount(1)
  }
  const initPlayers = settingsStore.playerCount;
  for (let i = 0; i < initPlayers; i++) {
    addNewPlayer()
  }
  isRowLayout.value = settingsStore.layout === 'row'
  isGridLayout.value = settingsStore.layout === 'grid'
  document.documentElement.addEventListener('contextmenu', (e) => {
    e.preventDefault()
    showMenuModal.value = true
  })
  window.addEventListener('resize', handleResize)
})

onBeforeUnmount(() => {
  document.documentElement.removeEventListener('contextmenu', (e) => {
    e.preventDefault()
    showMenuModal.value = true
  })
  if (state.scrollInterval) {
    cancelAnimationFrame(state.scrollInterval)
  }
  window.removeEventListener('resize', handleResize)
  playerStore.saveQueryState()
})

provide('showNotice', showNotice)

defineExpose({ 
  addNewPlayer,
  toggleLayout,
  toggleFiles,
  openSettingsModal,
  openSearchModal,
  openHelpModal
})
</script>

<template>
  <div v-if="showSetup" class="no-results-container">
    <FolderOpen class="w-16 h-16 text-muted-foreground mb-4 opacity-50" />
    <h2 class="text-xl font-bold mb-2">{{ t('wazoo.welcome') }}</h2>
    <p class="text-muted-foreground mb-4 text-center max-w-md">
      {{ t('wazoo.no_folders_msg') }}
    </p>
    
    <div v-if="appState.isScanning" class="flex flex-col items-center mb-4 mt-2">
      <Loader2 class="animate-spin mb-4 h-8 w-8 text-muted-foreground opacity-50" />
      <span class="text-sm text-muted-foreground">
        {{ appState.totalFiles === 0 ? t('wazoo.listing_files') : t('wazoo.loading_progress', { percent: appState.scanProgress, total: appState.totalFiles.toLocaleString() }) }}
      </span>
    </div>
    <button v-else class="retry-btn" @click="openSettingsModal">{{ t('wazoo.open_settings_to_add') }}</button>
  </div>
  
  <div v-if="appState.initialLoad && !showSetup" class="no-results-container">
    <Loader2 class="animate-spin w-16 h-16 text-muted-foreground mb-4 opacity-50" />
    <h2 class="text-xl font-bold mb-2">{{ t('common.loading') }}</h2>
    <p class="text-muted-foreground mb-4 text-center max-w-md">
      {{ t('wazoo.listing_files') }}
    </p>
  </div>

  <div v-if="showNoResults && !appState.initialLoad" class="no-results-container">
    <FileQuestion class="w-16 h-16 text-muted-foreground mb-4 opacity-50" />
    <h2 class="text-xl font-bold mb-2">{{ t('wazoo.no_files_found') }}</h2>
    <p class="text-muted-foreground mb-4">
      {{ t('wazoo.query_in_folder', { query: state.searchQuery, folder: state.searchFolder }) }}
    </p>
    <button class="retry-btn" @click="openSearchModal">{{ t('wazoo.try_another_search') }}</button>
  </div>

  <div class="players-grid" 
      :class="{ 
        'scroll-mode': isScrollMode, 
        'normal-mode': !isScrollMode,
        'row-layout': isRowLayout,
        'grid-layout': isGridLayout,
        [`players-${playerStore.players.length}`]: isGridLayout
      }">
    <div v-for="player in playerStore.players" 
        :key="player.id" 
        :data-player-id="player.id"
        class="player-container">
      <Player 
        :ref="(el) => playerStore.setupPlayerRef(el, player)"
        :id="player.id"
        :src="player.src"
        :codec="player.codec"
        :initial-time="player.initialTime"
        :initial-volume="player.initialVolume"
        :initial-muted="player.initialMuted"
        :scroll-mode="isScrollMode"
        @video-event="handleVideoEvent"
      />
    </div>
  </div>

  <div class="wazoo-controls" v-show="showWazooControls && !showSetup && !showNoResults && !appState.initialLoad">
    <FilePicker />
  </div>

  <SearchModal 
    ref="searchModalRef"
    @search="handleSearch"
    :is-open="showSearchModal" 
    :on-close="() => showSearchModal = false"
  />

  <MenuModal 
    :is-open="showMenuModal" 
    :on-close="() => showMenuModal = false"
    :wazoo-ref="{
      openSearchModal,
      addNewPlayer,
      toggleFiles,
      toggleLayout,
      openSettingsModal,
      openHelpModal
    }"
  />

  <SettingsModal 
    :is-open="showSettingsModal" 
    :on-close="() => showSettingsModal = false"
    @scan-start="() => { playerStore.clearVideos() }"
    @scan-end="() => { handleSearch(state.searchQuery, state.searchFolder) }"
  />

  <HelpModal 
    :is-open="showHelpModal" 
    :on-close="() => showHelpModal = false"
  />

  <Transition name="fade">
    <div v-if="showScanToast" class="scan-toast">
      {{ t('wazoo.added', { name: scanToastMessage }) }}
    </div>
  </Transition>

  <Notice ref="noticeRef" />
</template>

<style>
.no-results-container {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  z-index: 50;
  text-align: center;
  background: rgba(0, 0, 0, 0.7);
  padding: 40px;
  border-radius: 12px;
  backdrop-filter: blur(5px);
  border: 1px solid #333;
}

.retry-btn {
  background: #333;
  border: 1px solid #555;
  color: white;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.retry-btn:hover {
  background: #444;
  border-color: #666;
}

.text-muted-foreground {
  color: #888;
}

.mb-4 { margin-bottom: 1rem; }
.mb-2 { margin-bottom: 0.5rem; }
.text-xl { font-size: 1.25rem; }
.font-bold { font-weight: bold; }

.players-grid {
  display: flex;
  width: 100%;
}

.players-grid.normal-mode {
  position: static;
  flex-direction: column;
  overflow: visible;
  gap: 0;
  height: auto;
}

.players-grid.scroll-mode {
  position: relative;
  overflow: hidden;
  height: 100vh;
}

.normal-mode .player-container {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  width: 100%;
}

.scroll-mode .player-container {
  position: absolute;
  width: 100%;
  /* height: 50vh; */ /* Height is now dynamic */
  top: 0;
  left: 0;
}

.players-grid.row-layout {
  flex-direction: row !important;
}

.players-grid.grid-layout {
  display: grid !important;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 4px;
  width: 100%;
}

.grid-layout.players-1 {
  grid-template-columns: 1fr;
}

.grid-layout.players-2 {
  grid-template-columns: repeat(2, 1fr);
}

.grid-layout.players-3 {
  grid-template-columns: repeat(2, 1fr);
}
.grid-layout.players-3 .player-container:first-child {
  grid-column: span 2;
}

.grid-layout.players-4 {
  grid-template-columns: repeat(2, 1fr);
}

.grid-layout.players-5 {
  grid-template-columns: repeat(2, 1fr);
}

.grid-layout.players-6 {
  grid-template-columns: repeat(3, 1fr);
}

.grid-layout.players-7 {
  grid-template-columns: repeat(4, 1fr);
}

.grid-layout.players-8 {
  grid-template-columns: repeat(4, 1fr);
}

.grid-layout.players-9 {
  grid-template-columns: repeat(3, 1fr);
}

.grid-layout.players-10,
.grid-layout.players-11,
.grid-layout.players-12 {
  grid-template-columns: repeat(4, 1fr);
}

.grid-layout .player-container {
  aspect-ratio: 16/9;
  width: 100%;
  height: auto;
}

.scan-toast {
  position: fixed;
  bottom: 0;
  left: 0;
  width: 100%;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  text-align: center;
  padding: 8px 16px;
  z-index: 9999;
  font-size: 24px;
  backdrop-filter: blur(4px);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  border-radius: 8px;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>