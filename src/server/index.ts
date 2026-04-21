import { Hono } from 'hono'
import { cors } from 'hono/cors'
import { serve } from '@hono/node-server'
import { API_CONFIG } from './config'
import { findAvailablePort } from './utils'
import { routes } from './honoRouter'
import { log } from '@/lib/utils'
import { ffmpegManager } from '@/main/ffmpegManager'

export const createApiServer = async () => {
  const hono = new Hono()

  // 2. Add a variable to hold the server instance
  let serverInstance: any = null

  // Enable CORS
  hono.use(
    '*',
    cors({
      origin: '*',
      allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
      allowHeaders: ['Content-Type', 'Authorization'],
      exposeHeaders: ['Content-Length', 'X-Requested-With']
    })
  )

  // Try ports 3000-3100
  const port = await findAvailablePort(3000, 3100)
  API_CONFIG.updateConfig(port)

  hono.route('', routes)

  const startServer = () => {
    // 3. Capture the http.Server instance returned by serve()
    serverInstance = serve({
      fetch: hono.fetch,
      port,
      hostname: 'localhost'
    }, (info) => {
      log(`API server is running on ${API_CONFIG.baseUrl}`)
    })
  }

  // 4. Create a stop function that closes the server
  const stopServer = (): Promise<void> => {
    // We wrap this in a Promise because .close() is asynchronous
    return new Promise((resolve, reject) => {
      ffmpegManager.killAll(); // Hook into server stop
      if (serverInstance) {
        log('Closing API server...')
        serverInstance.close()
        resolve()
      } else {
        // If server isn't running, just resolve immediately
        resolve()
      }
    })
  }

  return {
    hono,
    start: startServer,
    stop: stopServer,
    port
  }
}