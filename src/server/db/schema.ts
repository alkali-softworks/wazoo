import {
  index,
  integer,
  sqliteTable,
  text,
  real
} from 'drizzle-orm/sqlite-core'
import { sql } from 'drizzle-orm'

export const VideoTable = sqliteTable('Video', {
  id: integer('id').primaryKey({ autoIncrement: true }),
  name: text('name').notNull(),
  path: text('path').notNull(),
  codec: text('codec').default('unknown'),
  width: integer('width').default(0),
  height: integer('height').default(0),
  duration: real('duration').default(0),
  has_subtitles: integer('has_subtitles', { mode: 'boolean' }).default(false),
  created_at: integer('created_at', { mode: 'timestamp' })
    .notNull()
    .default(sql`CURRENT_TIMESTAMP`)
}, (table) => {
  return {
    pathIdx: index('path_idx').on(table.path)
  }
})
export const schema = {
  tables: {
    Video: VideoTable
  }
}
