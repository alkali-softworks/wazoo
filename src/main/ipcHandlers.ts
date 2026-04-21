import { BrowserWindow, ipcMain, globalShortcut, dialog, app } from 'electron'
import { spawn, execSync, exec, execFile } from 'child_process'
import { eq } from 'drizzle-orm'
import { promisify } from 'util'
import { electronStore } from './store'
import { log } from '@/lib/utils'
import { Worker } from 'worker_threads'
import path from 'path'
import fs from 'fs'
import { searchVideos, getAllVideos, getVideoCount, clearVideos } from '@/server/db/models/videoModel'
import { ffmpegManager } from '@/main/ffmpegManager'

let currentScanId = 0

export function setupIpcHandlers(mainWindow: BrowserWindow) {

  ipcMain.handle('is-alt-pressed', (_, key) => {
    return !globalShortcut.isRegistered('Alt')
  })

  ipcMain.handle('get-settings', () => {
    return electronStore.get('settings')
  })

  ipcMain.handle('save-settings', (event, settings) => {
    electronStore.set('settings', settings)
  })

  ipcMain.handle('toggle-maximize-window', () => {
    if (mainWindow.isMaximized()) {
      mainWindow.unmaximize()
    } else {
      mainWindow.maximize()
    }
  })

  ipcMain.handle('minimize-window', () => {
    mainWindow.minimize()
  })

  ipcMain.handle('close-app', () => {
    mainWindow.close()
  })

  ipcMain.handle('get-window-bounds', () => {
    return mainWindow.getBounds()
  })

  ipcMain.handle('set-window-bounds', (_, bounds) => {
    mainWindow.setBounds(bounds)
  })

  ipcMain.handle('set-window-opacity', (_, opacity) => {
    log('set-window-opacity', opacity)
    const normalizedOpacity = Math.max(0, Math.min(1, Number(opacity)))
    mainWindow.setOpacity(normalizedOpacity)
  })

  ipcMain.handle('select-folder', async () => {
    const result = await dialog.showOpenDialog(mainWindow, {
      properties: ['openDirectory'],
      title: 'Select Media Folder'
    })

    if (!result.canceled && result.filePaths.length > 0) {
      return result.filePaths[0]
    }
    return null
  })

  ipcMain.handle('get-video-duration', async (_, filePath) => {
    try {
      // Use exec with a shorter timeout or just rely on the try/catch
      const ffprobePath = ffmpegManager.getFfprobePath()
      const { stdout } = await promisify(execFile)(ffprobePath, [
        '-v', 'error',
        '-show_entries', 'format=duration',
        '-of', 'default=noprint_wrappers=1:nokey=1',
        filePath
      ], { timeout: 5000 })
      return parseFloat(stdout.trim())
    } catch (error) {
      // Log only once or just return 0
      return 0
    }
  })

  ipcMain.handle('file-exists', async (_, filePath) => {
    try {
      await fs.promises.access(filePath, fs.constants.F_OK)
      return true
    } catch {
      return false
    }
  })


  ipcMain.handle('get-videos', async () => {
    try {
      const videos = await getAllVideos()
      return { success: true, videos }
    } catch (error) {
      console.error('Error getting videos:', error)
      return { success: false, error: error instanceof Error ? error.message : String(error) }
    }
  })

  ipcMain.handle('get-video-count', async () => {
    try {
      const count = await getVideoCount()
      return { success: true, count }
    } catch (error) {
      console.error('Error getting video count:', error)
      return { success: false, error: error instanceof Error ? error.message : String(error) }
    }
  })

  ipcMain.handle('check-video-subs', async (_, filePath: string) => {
    try {
      const db = (await import('@/server/db/connection')).default
      const { VideoTable } = await import('@/server/db/schema')
      
      const res = await db.select().from(VideoTable).where(eq(VideoTable.path, filePath)).limit(1)
      return { success: true, hasSubtitles: res[0]?.has_subtitles || false }
    } catch (error) {
      console.error('Error checking video subs:', error)
      return { success: false, hasSubtitles: false }
    }
  })

  ipcMain.handle('extract-subtitles', async (_, inputPath: string) => {
    try {
      const parsed = path.parse(inputPath)
      const outputPath = path.join(parsed.dir, `${parsed.name}.srt`)
      
      await ffmpegManager.extractSubtitles(inputPath, outputPath)
      return { success: true, outputPath }
    } catch (error) {
      console.error('Error extracting subtitles:', error)
      return { success: false, error: error instanceof Error ? error.message : String(error) }
    }
  })

  ipcMain.handle('search-videos', async (_, query: string, folder: string) => {
    try {
      const videos = await searchVideos(query, folder)
      return {
        success: true,
        videos: videos.map(v => ({
          path: v.path,
          codec: v.codec
        }))
      }
    } catch (error) {
      console.error('Error searching videos:', error)
      return { success: false, error: error instanceof Error ? error.message : String(error) }
    }
  })

  let scanWorker: Worker | null = null

  ipcMain.handle('scan-folders', async (event, clientScanId?: number) => {
    const scanId = clientScanId ?? ++currentScanId
    currentScanId = scanId
    log('starting scan', scanId)

    // Terminate existing worker if any
    if (scanWorker) {
      scanWorker.terminate()
      scanWorker = null
    }

    return new Promise((resolve) => {
      try {
        const settings = electronStore.get('settings')
        const folders = settings.mediaFolders || []

        if (folders.length < 1 || folders[0] === '') {
          log('no folders to scan');
          resolve({ success: false, error: 'No folders selected' })
          return
        }

        const userDataPath = app.getPath('userData')
        const dbPath = path.join(userDataPath, 'sqlite.db')

        // In production, we might need a different path for the worker script
        // For now assuming it's in the same directory as this file in build
        const workerPath = path.join(__dirname, 'scanner.js')

        const ffprobePath = ffmpegManager.getFfprobePath()

        scanWorker = new Worker(workerPath)

        scanWorker.on('message', (message) => {
          const { type, payload } = message
          if (payload.scanId !== scanId) return

          if (type === 'scan-progress') {
            const { processed, total, name } = payload
            event.sender.send('scan-progress', {
              processed,
              total,
              percent: total === 0 ? 100 : Math.round((processed / total) * 100),
              scanId,
              name
            })
          } else if (type === 'scan-complete') {
            scanWorker?.terminate()
            scanWorker = null
            resolve({ success: true })
          } else if (type === 'scan-error') {
            scanWorker?.terminate()
            scanWorker = null
            resolve({ success: false, error: payload.error })
          }
        })

        scanWorker.on('error', (err) => {
          console.error('Worker thread error:', err)
          scanWorker?.terminate()
          scanWorker = null
          resolve({ success: false, error: err instanceof Error ? err.message : String(err) })
        })

        scanWorker.postMessage({
          type: 'start-scan',
          payload: { scanId, folders, dbPath, ffprobePath }
        })

      } catch (error) {
        console.error('Error starting scan worker:', error)
        resolve({ success: false, error: error instanceof Error ? error.message : String(error) })
      }
    })
  })

  ipcMain.handle('convert-video', async (event, inputPath: string) => {
    return new Promise((resolve) => {
      try {
        const parsed = path.parse(inputPath)
        // Create the new highly-compatible file next to the original
        const outputPath = path.join(parsed.dir, `${parsed.name}_wazoo.mkv`)

        log(`🎬 Starting smart conversion for: ${parsed.name}`)

        // 1. Generate unique ID for the transcode process
        const id = `convert-${Date.now()}-${Math.random()}`

        // 2. Start transcode via ffmpegManager
        const child = ffmpegManager.startTranscode(id, inputPath, outputPath, (stderrData) => {
          // Send raw output back to Vue so it can parse it for the progress bar
          event.sender.send('convert-progress', { output: stderrData, filePath: inputPath })
        })

        // 3. Listen for completion or error
        child.on('close', (code) => {
          if (code === 0) {
            log(`✅ Conversion successful: ${outputPath}`)
            resolve({ success: true, newPath: outputPath })
          } else {
            log(`❌ FFmpeg failed with code ${code}`)
            resolve({ success: false, error: `FFmpeg exited with code ${code}` })
          }
        })

        child.on('error', (err) => {
          log(`Failed to start FFmpeg: ${err.message}`)
          resolve({ success: false, error: 'FFmpeg is missing from this system.' })
        })

      } catch (error) {
        resolve({ success: false, error: error instanceof Error ? error.message : String(error) })
      }
    })
  })
}
