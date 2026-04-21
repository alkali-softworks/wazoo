import { Context, Hono } from 'hono'
import {
  apiBadRequest,
  apiResponse,
  apiServerError
} from '@/server/utils/apiResponse'
import { electronStore } from '@/main/store'
import { log } from '@/lib/utils'

const routes = new Hono()

routes.get('/', async (c: Context) => {
  const settings = electronStore.get('settings')

  log('settings:', settings)

  //return apiResponse(c, store.path)
  return apiResponse(c, settings)
})

routes.post('/', async (c: Context) => {
  const body = await c.req.json()

  // Get current settings
  const currentSettings = electronStore.get('settings')

  // Merge new settings with current settings
  const newSettings = {
    ...currentSettings,
    lastQuery: body.lastQuery,
    lastFolder: body.lastFolder,
  }

  electronStore.set('settings', newSettings)

  const settings = electronStore.get('settings')
  return apiResponse(c, settings)
})

export default routes
