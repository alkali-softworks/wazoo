import { log } from '@/lib/utils'
import net from 'net'
import { app } from 'electron'
import path from 'path'
import fs from 'fs'
import { Context } from 'hono'

export const safeUrl = (url: string | undefined): string => {
  if (url == undefined) {
    return ''
  }

  return url.replace(/[^a-zA-Z0-9/-]/g, '')
}

export const getSlug = (name: string | undefined): string => {
  if (name == undefined) {
    return ''
  }

  let slug = name.toLowerCase().replace(/[^\w ]+/g, '').replace(/ +/g, '-')

  slug = slug.replace(/-+/g, '-')
  return slug
}

export const getUploadsPath = () => {
  const uploadsPath = path.join(app.getPath('userData'), 'uploads')

  // Ensure directory exists
  if (!fs.existsSync(uploadsPath)) {
    fs.mkdirSync(uploadsPath, { recursive: true })
  }

  try {
    fs.accessSync(uploadsPath, fs.constants.W_OK)
  } catch (error) {
    log('Uploads directory is not writable')
    return null
  }

  return uploadsPath
}

export const findAvailablePort = async (
  startPort: number,
  endPort: number
): Promise<number> => {
  for (let port = startPort; port <= endPort; port++) {
    try {
      await new Promise((resolve, reject) => {
        const server = net.createServer()
        server.unref()
        server.on('error', reject)
        server.listen(port, 'localhost', () => {
          server.close(() => resolve(port))
        })
      })
      return port
    } catch {
      continue
    }
  }
  throw new Error('No available ports found')
}

export const getMimeType = (fullPath: string) => {
  const MIME_TYPES: Record<string, string> = {
    '.js': 'application/javascript',
    '.css': 'text/css',
    '.jpg': 'image/jpeg',
    '.jpeg': 'image/jpeg',
    '.png': 'image/png',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.webp': 'image/webp',
    '.ico': 'image/x-icon',
    '.json': 'application/json',
    '.woff': 'font/woff',
    '.woff2': 'font/woff2',
    '.ttf': 'font/ttf',
    '.eot': 'application/vnd.ms-fontobject'
  }

  const ext = fullPath.substring(fullPath.lastIndexOf('.')).toLowerCase()
  const mimeType = MIME_TYPES[ext]

  if (!mimeType) return null

  return {
    'Content-Type': `${mimeType}; charset=utf-8`,
    'Cross-Origin-Resource-Policy': 'cross-origin'
  }
}

export const serveStaticFiles = async (
  c: Context,
  basePath: string,
  urlPrefix: string
) => {
  const filePath = c.req.path.replace(`/${urlPrefix}/`, '')
  const fullPath = path.join(basePath, urlPrefix, filePath)
  try {
    const file = await fs.promises.readFile(fullPath)
    const mimeType = getMimeType(fullPath)
    if (mimeType) {
      return c.newResponse(file, 200, mimeType)
    }
    return c.body(file)
  } catch (e) {
    return c.notFound()
  }
}
