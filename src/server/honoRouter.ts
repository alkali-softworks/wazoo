import { Context, Hono } from 'hono'
import { serveStaticFiles } from './utils'
import { app } from 'electron'
import path from 'path'
import fs from 'fs'

import settingsRoutes from './routes/settings'
import streamRoutes from './routes/stream'
import videoRoutes from './routes/video'

const routes = new Hono()

routes.route('/api/settings', settingsRoutes)
routes.route('/api/video', videoRoutes)
routes.route('', streamRoutes)

// Debug all paths
//log('App path:', app.getAppPath())
//log('Available files:', fs.readdirSync(app.getAppPath()))
//log('Vite files:', fs.readdirSync(join(app.getAppPath(), '.vite')))
// log('Renderer files:', fs.readdirSync(join(app.getAppPath(), '.vite/renderer')))
//log('Renderer files:', fs.readdirSync(join(app.getAppPath(), '.vite/renderer/main_window/')))
//log('uploads files:', fs.readdirSync(join(app.getAppPath(), '.vite/renderer/main_window/uploads')))

if (process.env.NODE_ENV === 'development') {
  routes.use('/*', async (c: Context) => {
    return c.redirect('http://localhost:5173')
  })
} else {
  const rendererPath = path.join(
    app.getAppPath(),
    '.vite/renderer/main_window'
  )

  // Serve static assets explicitly
  //routes.get('/assets/*', (c) => serveStaticFiles(c, rendererPath, 'assets'))
  //routes.get('/images/*', (c) => serveStaticFiles(c, rendererPath, 'images'))
  //routes.get('/uploads/*', (c) => serveStaticFiles(c, rendererPath, 'uploads'))

  // Serve the SPA
  routes.get('/', async (c: Context) => {
    const indexPath = path.join(rendererPath, 'index.html')
    const html = await fs.promises.readFile(indexPath, 'utf-8')
    return c.html(html)
  })
}

export { routes }
