import { app, BrowserWindow, Menu } from 'electron'
import path from 'node:path'
import started from 'electron-squirrel-startup'
import { setupIpcHandlers } from '@/main/ipcHandlers'
import { electronStore } from '@/main/store'
import { client } from '@/server/db/connection'
import { ffmpegManager } from '@/main/ffmpegManager'


declare const MAIN_WINDOW_VITE_DEV_SERVER_URL: string | undefined
declare const MAIN_WINDOW_VITE_NAME: string
let mainWindow: BrowserWindow | null = null

let isQuitting = false

function log(...args: any[]) {
  const isDev = process.env.NODE_ENV === 'development'
  if (isDev) {
    console.log(...args)
  }
}

export function createMainWindow() {
  const settings = electronStore.get('settings')
  //log('settings:', settings)

  const { width, height } = settings.windowBounds

  const mainWindow = new BrowserWindow({
    width: width,
    height: height,
    frame: false,
    transparent: true,
    backgroundColor: '#000000',
    hasShadow: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
      webSecurity: false,  // Add this line - but be aware of security implications
      allowRunningInsecureContent: true  // Add this line too
    }
  })

  const normalizedOpacity = Math.max(0, Math.min(1, Number(settings.windowOpacity)))
  mainWindow.setOpacity(normalizedOpacity)

  // Save window size when it's resized
  function saveBounds() {
    const bounds = mainWindow.getBounds()
    electronStore.set('settings.windowBounds', bounds)
  }
  mainWindow.on('resize', saveBounds)
  mainWindow.on('move', saveBounds)

  mainWindow.on('close', () => {
    app.quit()
  })



  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL)
  } else {
    mainWindow.loadFile(
      path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`)
    )
  }

  if (process.env.NODE_ENV === 'development') {
    mainWindow.webContents.openDevTools()
  }

  setupSecurityHeaders(mainWindow)
  return mainWindow
}

function setupSecurityHeaders(mainWindow: BrowserWindow) {
  mainWindow.webContents.session.webRequest.onHeadersReceived(
    (details, callback) => {
      const responseHeaders = {
        ...details.responseHeaders,
        'Content-Security-Policy': [
          `default-src 'self' file:; 
           script-src 'self'; 
           style-src 'self' 'unsafe-inline'; 
           connect-src 'self' * file:;
           img-src 'self' blob: * file:;
           media-src 'self' * file:;`
        ]
      }

      // Only add CORS headers for local requests
      // if (details.url.includes('192.168.1.100')) {
      //   responseHeaders['Access-Control-Allow-Origin' as keyof typeof responseHeaders] = ['*']
      // }

      callback({ responseHeaders })
    }
  )
}

if (started) {
  app.quit()
}

app.whenReady().then(async () => {




  Menu.setApplicationMenu(null)
  mainWindow = createMainWindow()

  setupIpcHandlers(mainWindow)
})

app.on('before-quit', async (event) => {
  if (isQuitting) {
    return
  }

  event.preventDefault()
  // Prevent the app from quitting immediately to allow for cleanup
  ffmpegManager.killAll()

  try {
    log('Closing DB before quit...')
    client.close()
  } catch (err) {
    console.error('Failed to close DB:', err)
  }

  // Initiate a graceful app quit
  setTimeout(() => {
    isQuitting = true
    app.quit()
  }, 500)
})

app.on('window-all-closed', () => {
  app.quit()
})

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    mainWindow = createMainWindow()
  }
})
