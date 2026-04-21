import Store from 'electron-store'

const defaults = {
  settings: {
    defaultVolume: 1.0,
    lastQuery: '',
    lastFolder: '',
    playerCount: 1,
    layout: 'column',
    windowBounds: {
      width: 800,
      height: 640
    },
    windowOpacity: 1.0,
    queryStates: {},
    mediaFolders: ['']
  }
}

export const electronStore = new Store({
  defaults
}) as any

// Initialize store with defaults for any missing values
Object.entries(defaults.settings).forEach(([key, value]) => {
  if (!electronStore.has(`settings.${key}`)) {
    electronStore.set(`settings.${key}`, value)
  }
})
