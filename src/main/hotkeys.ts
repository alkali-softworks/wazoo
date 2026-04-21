import { BrowserWindow, globalShortcut } from 'electron'
import { log } from '@/lib/utils'

// THESE ARE GLOBAL SHORTCUTS (not just when the app is in focus)

// Constants for commands
const COMMANDS = {
  TOGGLE_PLAY: 'player-toggle-play',
  NEXT_TRACK: 'player-next',
  VOLUME_UP: 'player-volume-up',
  VOLUME_DOWN: 'player-volume-down'
} as const

type CommandKeys = keyof typeof COMMANDS

// Constants for shortcuts
const SHORTCUTS = {
  TOGGLE_PLAY: ['CommandOrControl+Alt+P', 'MediaPlayPause'],
  NEXT_TRACK: ['MediaNextTrack'],
  VOLUME_UP: [']'],
  VOLUME_DOWN: ['[']
} as const

export function registerHotkeys(mainWindow: BrowserWindow): void {
  try {
    // Register all shortcuts
    ;(Object.entries(SHORTCUTS) as [CommandKeys, readonly string[]][]).forEach(
      ([command, shortcuts]) => {
        shortcuts.forEach((shortcut) => {
          if (globalShortcut.isRegistered(shortcut)) {
            console.warn(`Shortcut ${shortcut} is already registered`)
            return
          }

          try {
            globalShortcut.register(shortcut, () => {
              // Send directly to the specific command channel instead of 'player-control'
              mainWindow.webContents.send(COMMANDS[command], 'focused')
              log(
                `Hotkey pressed: ${shortcut}, sending command: ${
                  COMMANDS[command]
                }`
              )
            })

            log(`Registered shortcut: ${shortcut}`)
          } catch (error) {
            console.error(`Failed to register shortcut: ${shortcut}`, error)
          }
        })
      }
    )
  } catch (error) {
    console.error('Error registering hotkeys:', error)
  }
}

export function unregisterHotkeys(): void {
  try {
    // Get all shortcuts
    const allShortcuts = Object.values(SHORTCUTS).flat()

    // Unregister specific shortcuts
    allShortcuts.forEach((shortcut) => {
      globalShortcut.unregister(shortcut)
    })
  } catch (error) {
    console.error('Error unregistering hotkeys:', error)
    // Fallback to unregister all as a safety measure
    globalShortcut.unregisterAll()
  }
}

export function checkHotkeys(): boolean {
  const allShortcuts = Object.values(SHORTCUTS).flat()
  return allShortcuts.every((shortcut) => globalShortcut.isRegistered(shortcut))
}
