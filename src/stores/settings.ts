import { defineStore } from 'pinia'
import { toRaw } from 'vue'
import { ISettings } from '@/types'
import api from '@/lib/apiClient'
import { log } from '@/lib/utils'

export const useSettingsStore = defineStore('settings', {
  state: (): ISettings & { _modified: Set<string> } => ({
    runServer: false,
    defaultVolume: 1.0,
    lastQuery: null,
    lastFolder: null,
    playerCount: 1,
    layout: 'column',
    windowBounds: {
      width: 800,
      height: 1000
    },
    windowOpacity: 1.0,
    queryStates: {},
    mediaFolders: [],
    language: 'en',
    _modified: new Set()
  }),

  actions: {
    async loadSettings(): Promise<void> {
      let settings: ISettings

      if (window.electron) {
        settings = await window.electron.invoke('get-settings')
      } else {
        settings = await api.get('/settings')
      }
      log('settings', settings)

      this.$patch(settings)
      this._modified.clear() // Reset modified tracking after load
    },

    async saveSettings(): Promise<void> {
      try {
        // Don't save if nothing was modified
        if (this._modified.size === 0) return

        // First get current clean settings from storage
        let cleanSettings: ISettings
        if (window.electron) {
          cleanSettings = await window.electron.invoke('get-settings') as ISettings
        } else {
          cleanSettings = await api.get('/settings') as ISettings
        }

        // Only update the modified fields
        Array.from(this._modified).forEach((field) => {
          const settingKey = field as keyof ISettings;
          (cleanSettings as any)[settingKey] = this[settingKey];
        });

        // Create a serializable copy of settings without the _modified Set
        const settingsToSave = JSON.parse(JSON.stringify(toRaw(cleanSettings)))
        //log('settingsToSave', settingsToSave)

        // Save the complete settings object with modified fields updated
        if (window.electron) {
          await window.electron.invoke('save-settings', settingsToSave)
        } else {
          await api.post('/settings', settingsToSave)
        }

        // Clear modified flags after successful save
        this._modified.clear()
      } catch (error) {
        console.error('Error saving settings:', error)
      }
    },

    hashQuery(query: string): string {
      let hash = 0
      for (let i = 0; i < query.length; i++) {
        const char = query.charCodeAt(i)
        hash = ((hash << 5) - hash) + char
        hash = hash & hash
      }
      return Math.abs(hash).toString(16)
    },

    getQueryState(query: string): { src: string, time: number } | undefined {
      const queryHash = this.hashQuery(query)
      const stateStr = this.queryStates[queryHash]

      if (!stateStr) return undefined

      const [src, timeStr] = stateStr.split('|')
      return {
        src,
        time: parseInt(timeStr, 10)
      }
    },

    async saveQueryState(query: string, videoSrc: string, currentTime: number): Promise<void> {
      const queryHash = this.hashQuery(query)
      const stateStr = `${videoSrc}|${Math.floor(currentTime)}`
      this.queryStates[queryHash] = stateStr
      this._modified.add('queryStates')
      await this.saveSettings()
    },

    async setMediaFolders(folders: string[]): Promise<void> {
      this.mediaFolders = folders
      this._modified.add('mediaFolders')
      await this.saveSettings()
    },

    async addMediaFolder(folder: string): Promise<void> {
      if (!this.mediaFolders.includes(folder)) {
        this.mediaFolders = [...this.mediaFolders, folder]
        this._modified.add('mediaFolders')
        await this.saveSettings()
      }
    },

    async removeMediaFolder(folder: string): Promise<void> {
      this.mediaFolders = this.mediaFolders.filter((f: string) => f !== folder)
      this._modified.add('mediaFolders')
      await this.saveSettings()
    },

    async setVolume(volume: number): Promise<void> {
      this.defaultVolume = Math.max(0, Math.min(1, volume))
      this._modified.add('defaultVolume')
      await this.saveSettings()
    },

    async setWindowBounds(bounds: { width: number; height: number }): Promise<void> {
      this.windowBounds = bounds
      this._modified.add('windowBounds')
      await this.saveSettings()
    },

    async setWindowOpacity(v: number): Promise<void> {
      this.windowOpacity = v
      this._modified.add('windowOpacity')
      await this.saveSettings()
    },

    async setRunServer(v: boolean): Promise<void> {
      this.runServer = v
      this._modified.add('runServer')
      await this.saveSettings()
    },

    async setLastQuery(query: string | null, folder: string | string[] | null): Promise<void> {
      this.lastQuery = query
      this.lastFolder = folder
      this._modified.add('lastQuery')
      this._modified.add('lastFolder')
      await this.saveSettings()
    },

    async setPlayerCount(count: number): Promise<void> {
      this.playerCount = count
      this._modified.add('playerCount')
      await this.saveSettings()
    },

    async setLayout(layout: ISettings['layout']): Promise<void> {
      this.layout = layout
      this._modified.add('layout')
      await this.saveSettings()
    },
    async setLanguage(lang: ISettings['language']): Promise<void> {
      this.language = lang
      this._modified.add('language')
      await this.saveSettings()
    }
  }
})
