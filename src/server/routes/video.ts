import { Context, Hono } from 'hono'
import { apiBadRequest, apiResponse } from '@/server/utils/apiResponse'

import { getAllVideos, searchVideos } from '../db/models/videoModel'
const app = new Hono()

app.get('/', async (c: Context) => {
  const videos = await getAllVideos()

  return apiResponse(c, videos)
})

app.get('/search', async (c: Context) => {

  const query = c.req.query('q')
  const folder = c.req.query('folder')

  if (!query || query == '') {
    return apiBadRequest(c, 'q is required')
  }

  // if (!folder || folder == '') {
  //   return apiBadRequest(c, 'folder is required')
  // }

  const videos = await searchVideos(query, folder || 'All')

  return apiResponse(c, videos)
})

export default app
