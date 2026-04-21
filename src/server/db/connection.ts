import Database from 'better-sqlite3'
import { drizzle } from 'drizzle-orm/better-sqlite3'
import path from 'path'
import fs from 'fs'
import { app } from 'electron'
import { schema } from './schema'

const isDev = process.env.NODE_ENV === 'development'

const defaultDbPath = isDev
  ? './default.db'
  : path.join(process.resourcesPath, 'default.db')

const userDataPath = app.getPath('userData')
const userDbPath = path.join(userDataPath, 'sqlite.db')

if (!fs.existsSync(userDbPath)) {
  if (fs.existsSync(defaultDbPath)) {
    fs.copyFileSync(defaultDbPath, userDbPath)
  }
}

export const client = new Database(userDbPath)

const db = drizzle(client, { schema })

export default db
