import { parentPort } from 'worker_threads'
import path from 'path'
import { promises as fs } from 'fs'
import Database from 'better-sqlite3'
import { drizzle } from 'drizzle-orm/better-sqlite3'
import { VideoTable } from '../server/db/schema'
import { execFile } from 'child_process'
import { promisify } from 'util'

const execFileAsync = promisify(execFile)

const videoExtensions = new Set(['.mkv', '.mp4', '.avi', '.mov', '.webm'])

function isVideoFile(filename: string): boolean {
  const ext = path.extname(filename).toLowerCase()
  return videoExtensions.has(ext)
}

export function cleanVideoName(filename: string): string {
  const cleaned = filename
    .replace(/\[.*?\]/g, ' ')
    .replace(/[\._]/g, ' ')
    .replace(/\s{2,}/g, ' ')
    .replace(/^[\-\s]+|[\-\s]+$/g, '')
    .trim()

  return cleaned || filename
}

// Sniff the exact codec and profile using ffprobe
interface VideoMetadata {
  codec: string;
  width: number;
  height: number;
  duration: number;
  hasSubtitles: boolean;
}

let isFfprobeAvailable = true;

// Sniff codec, profile, dimensions, and length in a single fast pass
async function getVideoMetadata(filePath: string, ffprobePath: string): Promise<VideoMetadata> {

  const fallback = { codec: 'unknown', width: 0, height: 0, duration: 0, hasSubtitles: false }

  if (!isFfprobeAvailable) return fallback;

  console.log(`Probing metadata for ${filePath}`)

  try {
    const { stdout } = await execFileAsync(ffprobePath, [
      '-v', 'error',
      '-analyzeduration', '100000',
      '-probesize', '5000000',
      // Ask for stream details (video and subtitle) AND container format details
      '-show_entries', 'stream=codec_name,profile,width,height,codec_type:format=duration',
      '-of', 'json',
      filePath
    ])

    const data = JSON.parse(stdout)
    const videoStream = data.streams?.find((s: any) => s.codec_type === 'video') || {}
    const hasSubtitles = data.streams?.some((s: any) => s.codec_type === 'subtitle') || false
    const format = data.format || {}

    const codecRaw = (videoStream.codec_name || '').toLowerCase()
    const profileRaw = (videoStream.profile || '').toLowerCase()

    let codec = 'unknown'
    if (codecRaw.includes('hevc') && profileRaw.includes('main 10')) codec = 'hevc-10bit'
    else if (codecRaw.includes('hevc')) codec = 'hevc-8bit'
    else if (codecRaw.includes('h264')) codec = 'h264'
    else if (codecRaw.includes('av1')) codec = 'av1'
    else if (codecRaw.includes('vp9')) codec = 'vp9'

    return {
      codec,
      width: parseInt(videoStream.width) || 0,
      height: parseInt(videoStream.height) || 0,
      // Fallback to stream duration if format duration is missing
      duration: parseFloat(format.duration || videoStream.duration) || 0,
      hasSubtitles
    }
  } catch (err) {
    console.error(`Failed to probe metadata for ${filePath}.`, err)
    return fallback
  }
}

async function scanDirectory(dirPath: string, scanId: number, currentScanId: { value: number }): Promise<{ name: string, path: string }[]> {
  const videos: { name: string, path: string }[] = []
  const queue: string[] = [dirPath]

  while (queue.length > 0) {
    if (currentScanId.value !== scanId) return videos
    const dir = queue.shift()!

    try {
      const files = await fs.readdir(dir, { withFileTypes: true })

      for (const file of files) {
        if (currentScanId.value !== scanId) return videos
        const fullPath = path.join(dir, file.name)

        if (file.isDirectory()) {
          queue.push(fullPath)
        } else if (isVideoFile(file.name)) {
          const nameWithoutExt = path.parse(file.name).name
          videos.push({
            name: cleanVideoName(nameWithoutExt),
            path: fullPath
          })
        }
      }
    } catch (err) {
      console.error(`Error scanning directory ${dir}:`, err)
    }
  }

  return videos
}

// Updated to use a fixed concurrency limit to avoid fork bombing the host OS causing SIGINT kills randomly.
async function processConcurrently<T>(
  items: T[],
  concurrencyLimit: number,
  processor: (item: T) => Promise<void>,
  onProgress?: (processed: number, total: number, item?: T) => void,
  scanId?: number,
  currentScanId?: { value: number }
) {
  const total = items.length

  if (total === 0) {
    onProgress?.(0, 0)
    return
  }

  let processedCount = 0
  let index = 0

  const workers = Array(Math.min(concurrencyLimit, total)).fill(null).map(async () => {
    while (index < total) {
      if (scanId && currentScanId && currentScanId.value !== scanId) return

      const currentIndex = index++
      const item = items[currentIndex]

      await processor(item)
      processedCount++
      onProgress?.(processedCount, total, item)

      // Let worker thread event loops clear pending cancel IPCs
      await new Promise(resolve => setTimeout(resolve, 1))
    }
  })

  await Promise.all(workers)
}

if (parentPort) {
  let currentScanId = { value: 0 }
  let db: any = null

  parentPort.on('message', async (message) => {
    const { type, payload } = message

    if (type === 'start-scan') {
      const { scanId, folders, dbPath, ffprobePath } = payload
      currentScanId.value = scanId

      try {
        if (!db) {
          const client = new Database(dbPath)
          db = drizzle(client)
        }

        // Clear existing videos
        db.delete(VideoTable).run()

        // Check if ffprobe is valid before starting
        try {
          await execFileAsync(ffprobePath || 'ffprobe', ['-version'])
          isFfprobeAvailable = true
          console.log(`ffprobe found at: ${ffprobePath || 'ffprobe'}`)
        } catch (err) {
          isFfprobeAvailable = false
          console.warn(`ffprobe NOT FOUND at: ${ffprobePath || 'ffprobe'}. Metadata probing will be disabled.`, err)
        }

        let allVideos: { name: string, path: string }[] = []

        for (const folder of folders) {
          if (currentScanId.value !== scanId) return
          const videos = await scanDirectory(folder, scanId, currentScanId)
          allVideos = allVideos.concat(videos)
        }

        if (currentScanId.value !== scanId) return

        await processConcurrently(
          allVideos,
          8, // Safer limit: max 8 concurrent ffprobe process spawning blocks
          async (video) => {
            // Sniff the codec right before inserting into SQLite
            const metadata = await getVideoMetadata(video.path, ffprobePath || 'ffprobe')

            db.insert(VideoTable).values({
              name: video.name,
              path: video.path,
              codec: metadata.codec,
              width: metadata.width,
              height: metadata.height,
              duration: metadata.duration,
              has_subtitles: metadata.hasSubtitles
            }).run()
          },
          (processed, total, video) => {
            if (currentScanId.value !== scanId) return
            parentPort?.postMessage({
              type: 'scan-progress',
              payload: { processed, total, scanId, name: video?.name }
            })
          },
          scanId,
          currentScanId
        )

        if (currentScanId.value === scanId) {
          parentPort?.postMessage({ type: 'scan-complete', payload: { scanId } })
        }
      } catch (error) {
        console.error('Worker error:', error)
        parentPort?.postMessage({
          type: 'scan-error',
          payload: { error: error instanceof Error ? error.message : String(error), scanId }
        })
      }
    } else if (type === 'cancel-scan') {
      currentScanId.value = payload.scanId
    }
  })
}