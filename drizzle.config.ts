import { defineConfig } from 'drizzle-kit'

export default defineConfig({
  out: './src/server/db/drizzle',
  schema: './src/server/db/schema.ts',
  dialect: 'sqlite',
  dbCredentials: { url: 'file:./default.db' }
})
