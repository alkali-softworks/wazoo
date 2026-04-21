import db from '@/server/db/connection'
import { eq } from 'drizzle-orm'

export default async <T>(table: any, values: object) => {
  const insertRes = await db.insert(table).values(values)

  if (insertRes && insertRes.lastInsertRowid) {
    const res = await db.select().from(table)
      .where(eq(table.id, insertRes.lastInsertRowid))
      .limit(1)

    if (res.length > 0) {
      return res[0] as T
    }
  }

  return null
}
