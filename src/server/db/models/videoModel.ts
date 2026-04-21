import db from '@/server/db/connection'
import { VideoTable } from '@/server/db/schema'
import { and, or, not, like, sql } from 'drizzle-orm'
import { log } from '@/lib/utils'

export async function searchVideos(queryStr: string, folder: string | string[]) {

  log(`searchVideos: "${folder}", "${queryStr}"`)

  // Split into clauses
  const clauses = queryStr.toLowerCase().split(',').map(c => c.trim()).filter(c => c.length > 0)

  const queryConditions = clauses.map(clause => {
    if (clause.startsWith('not ')) {
      // Handle negative clause
      const term = clause.replace('not ', '').trim()
      const wildcardPattern = `%${term.split(/\s+/).join('%')}%`
      return not(like(VideoTable.path, wildcardPattern))
    } else {
      // Handle positive clause
      const wildcardPattern = `%${clause.split(/\s+/).join('%')}%`
      return like(VideoTable.path, wildcardPattern)
    }
  })

  const finalConditions = []

  if (queryConditions.length > 0) {
    finalConditions.push(or(...queryConditions))
  }

  if (folder) {
    if (Array.isArray(folder)) {
      if (folder.length > 0 && !folder.includes('All')) {
        const folderConditions = folder.map(f => like(VideoTable.path, `${f}%`))
        finalConditions.push(or(...folderConditions))
      }
    } else if (folder !== 'All') {
      finalConditions.push(like(VideoTable.path, `${folder}%`))
    }
  }

  const res = await db
    .select()
    .from(VideoTable)
    .where(finalConditions.length > 0 ? and(...finalConditions) : undefined)
    .orderBy(VideoTable.path)

  //log('searchVideos', res)

  return res
}

export async function getVideoCount() {
  const result = await db
    .select({
      count: sql<number>`count(*)`
    })
    .from(VideoTable)

  return result[0].count
}

export async function getAllVideos() {
  return await db
    .select()
    .from(VideoTable)
    .orderBy(VideoTable.name)
}

export async function addVideo(name: string, path: string) {
  //log('addVideo', name, path)
  return await db.insert(VideoTable).values({
    name,
    path
  }).returning()
}

export async function clearVideos() {
  return await db.delete(VideoTable)
}