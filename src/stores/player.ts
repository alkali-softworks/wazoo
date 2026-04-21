import { defineStore } from 'pinia'
import { PlayerMeta } from '@/types/index'
import { log } from '@/lib/utils'
import Player from '@/components/Player.vue'
import { useSettingsStore } from '@/stores/settings'
import { useAppState } from '@/stores/app'

export enum PlayMode {
  RANDOM = 'random',
  SEQUENTIAL = 'sequential'
}

interface PlayerStoreState {
  focusedPlayerId: number | null
  players: PlayerMeta[]
  videos: string[]
  codecs: string[]
}

export const usePlayerStore = defineStore('player', {
  state: (): PlayerStoreState => ({
    focusedPlayerId: null,
    players: [],
    videos: [],
    codecs: []
  }),

  getters: {
    currentPlayer: (state): PlayerMeta => {
      const player = state.players.find((player) => player.id === state.focusedPlayerId)
      if (player) {
        return player
      } else {
        return state.players[0]
      }
    },

    playerCount: (state): number => { return state.players.length },
  },

  actions: {
    addPlayer(): PlayerMeta {
      const newId = this.generateUniqueId();
      const player: PlayerMeta = {
        id: newId,
        src: '',
        initialTime: 0,
        initialVolume: 1,
        initialMuted: true,
        playHistory: [],
        forwardHistory: [],
        playProgress: new Map(),
        currentHistoryIndex: -1,
        playMode: PlayMode.RANDOM,
        lastPlayedIndex: -1,
        codec: ''
      }
      this.players.push(player)
      this.playNextVideo(newId)
      return player
    },

    removePlayer(playerId: number) {
      this.players = this.players.filter((player: PlayerMeta) =>
        player.id !== playerId
      )

      // If the removed player was focused, focus the first remaining player
      if (this.focusedPlayerId === playerId) {
        const firstRemainingPlayer = this.players[0]
        if (firstRemainingPlayer) {
          this.setFocusedPlayer(firstRemainingPlayer.id)
        }
      }
    },

    getFocusedPlayer(): InstanceType<typeof Player> {
      if (this.currentPlayer) {
        return this.currentPlayer.ref
      } else {
        return this.players[0].ref
      }
    },

    setNextPlayerFocused() {
      // Get current index and increment
      const currentIndex = this.players.findIndex((p: PlayerMeta) => p.id === this.focusedPlayerId)
      const nextIndex = (currentIndex + 1) % this.players.length
      const nextPlayer = this.players[nextIndex]

      this.setFocusedPlayer(nextPlayer.id)
      return
    },

    generateUniqueId(): number {
      const timestamp = Date.now()
      return timestamp + Math.floor(Math.random() * 1000)
    },

    setupPlayerRef(el: any, player: PlayerMeta) {
      if (el) {
        player.ref = el as InstanceType<typeof Player>
      }
    },

    getPlayerRef(playerId: number): InstanceType<typeof Player> | undefined {
      const player = this.getPlayerById(playerId)
      if (!player) {
        log('getPlayerRef: no player found for id:', playerId)
        return
      }
      return player?.ref
    },

    setFocusedPlayer(id: number): void {
      //log('Setting focused player to:', id)
      this.focusedPlayerId = id
    },

    isFocusedPlayer(id: number): boolean {
      return this.focusedPlayerId === id
    },

    getPlayerById(id: number): PlayerMeta | undefined {
      return this.players.find((player: PlayerMeta) => player.id === id)
    },

    getCurrentVideo(playerId: number): string | null {
      const player = this.getPlayerById(playerId)
      return player ? player.src : null
    },

    addVideo(src: string): void {
      this.videos.push(src)
    },

    addVideos(videos: string[], codecs: string[]): void {
      this.videos.push(...videos)
      this.codecs.push(...codecs)

      // Update players: Only change video if the current one is NO LONGER in the list
      this.players.forEach((player: PlayerMeta) => {
        const currentVideoStillExists = player.src && this.videos.includes(player.src)

        if (!currentVideoStillExists) {
          // If the video is gone (or player was empty), pick a new one
          this.playNextVideo(player.id)
        } else if (player.src) {
          // Sync codec if it's missing or wrong
          const index = this.videos.indexOf(player.src)
          if (index !== -1) {
            player.codec = this.codecs[index] || ''
          }
        }
      })
    },

    addCodecs(codecs: string[]): void {
      this.codecs.push(...codecs)
    },

    clearVideos(): void {
      this.videos = []
      this.codecs = []
    },

    getNextVideo(playerId: number): string | null {
      const player = this.getPlayerById(playerId)
      if (!player || this.videos.length === 0) {
        log('no videos to play')
        return null
      }

      //log('history index', player.currentHistoryIndex)

      if (player.playMode === PlayMode.RANDOM) {
        const index = Math.floor(Math.random() * this.videos.length)
        player.lastPlayedIndex = index
        return this.videos[index]
      } else {
        // Sequential mode
        let nextIndex = player.lastPlayedIndex + 1

        // If we're at the end, start from beginning
        if (nextIndex >= this.videos.length) {
          nextIndex = 0
        }

        player.lastPlayedIndex = nextIndex
        return this.videos[nextIndex]
      }

      log('no compatible videos found')
      return null
    },

    playNextVideo(playerId: number): void {
      const player = this.getPlayerById(playerId) as PlayerMeta
      if (!player) return

      // Save current progress
      this.savePlayerProgress(player)

      // Check if we have videos in forward history
      if (player.forwardHistory.length > 0) {
        this.playForwardHistory(player)
        return
      }

      log('No forward history, getting next video normally')
      // No forward history, get next video normally
      const newSrc = this.getNextVideo(playerId)
      if (!newSrc) return

      // Clear forward history when getting a new video
      player.forwardHistory = []

      // Add current video to history before changing to new one
      if (player.src) {
        player.playHistory.push(player.src)
      }

      player.initialTime = 0 // New videos start from beginning
      player.src = newSrc

      // Update codec
      const index = this.videos.indexOf(newSrc)
      if (index !== -1) {
        player.codec = this.codecs[index] || ''
      }
    },

    playPrevious(playerId: number): void {
      const player = this.getPlayerById(playerId) as PlayerMeta

      if (!player || player.playHistory.length === 0) {
        log('no history')
        return
      }

      // Save current progress
      this.savePlayerProgress(player)

      // Get the previous video from history
      const previousVideo = player.playHistory.pop()
      if (previousVideo) {
        // Add current video to forward history
        if (player.src) {
          player.forwardHistory.unshift(player.src)
        }

        // Set the video
        this.restorePlayerProgress(player, previousVideo)
        player.src = previousVideo

        // Update lastPlayedIndex and codec
        const newIndex = this.videos.indexOf(previousVideo)
        if (newIndex !== -1) {
          player.lastPlayedIndex = newIndex
          player.codec = this.codecs[newIndex] || ''
        }
      }
    },

    savePlayerProgress(player: PlayerMeta): void {
      if (!player.src) {
        //log('player.src not available in savePlayerProgress')
        return
      }

      if (!player.ref) {
        log('player.ref not available in savePlayerProgress')
        return
      }

      log('Saving progress', player.src, player.ref.getCurrentTime())
      //this.saveQueryState()

      player.playProgress.set(player.src, Math.floor(player.ref.getCurrentTime()))

      // Remove save progress if we are near the end
      if (player.ref.getDuration() - player.ref.getCurrentTime() < 45) {
        player.playProgress.delete(player.src)
      }
    },

    restorePlayerProgress(player: PlayerMeta, src: string): void {
      const savedProgress = player.playProgress.get(src)
      player.initialTime = savedProgress !== undefined ? Math.floor(savedProgress) : 0
    },

    playForwardHistory(player: PlayerMeta): void {
      // Get next video from forward history
      const nextVideo = player.forwardHistory[0]
      log('Playing from forward history:', nextVideo)

      // Move current video to play history if it exists
      if (player.src) {
        player.playHistory.push(player.src)
      }

      // Remove video from forward history
      player.forwardHistory.shift()

      // Set the video
      this.restorePlayerProgress(player, nextVideo)
      player.src = nextVideo

      // Update lastPlayedIndex and codec
      const newIndex = this.videos.indexOf(nextVideo)
      if (newIndex !== -1) {
        player.lastPlayedIndex = newIndex
        player.codec = this.codecs[newIndex] || ''
      }
    },

    togglePlayMode(playerId: number): void {
      const player = this.getPlayerById(playerId)
      if (player) {
        player.playMode = player.playMode === PlayMode.RANDOM ?
          PlayMode.SEQUENTIAL :
          PlayMode.RANDOM
        log(`Switched player ${playerId} to ${player.playMode} mode`)
      }
    },

    loadQueryState(): void {
      const appState = useAppState()
      const settingsStore = useSettingsStore()

      if (!appState.initialLoad) {
        return
      }
      appState.setInitialLoad(false)

      const savedState = settingsStore.getQueryState(appState.query)
      if (savedState && this.players.length > 0) {
        // Make sure file exists in the present query
        if (!this.videos.includes(savedState.src)) {
          log('Query state not found in current query')
          return
        }

        const firstPlayer = this.players[0]
        firstPlayer.initialTime = savedState.time
        firstPlayer.src = savedState.src
        
        // Sync codec
        const index = this.videos.indexOf(savedState.src)
        if (index !== -1) {
          firstPlayer.codec = this.codecs[index] || ''
        }
        
        log('LOADING SAVEDSTATE:', savedState.src, savedState.time)

        // Restore player position so that the next load starts normally
        setTimeout(() => {
          firstPlayer.initialTime = 0
        }, 100)
      }
    },

    saveQueryState(): void {
      const appState = useAppState()
      const settingsStore = useSettingsStore()

      if (!appState.query || !this.currentPlayer?.src) return
      //log('Saving query state')

      const currentTime = this.currentPlayer.ref?.getCurrentTime() || 0
      settingsStore.saveQueryState(
        appState.query,
        this.currentPlayer.src,
        Math.floor(currentTime)
      )
    }
  }
})
