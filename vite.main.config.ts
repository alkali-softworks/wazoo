import { defineConfig } from 'vite'
import path from 'path'

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  build: {
    rollupOptions: {
      external: [
        'electron',
        'better-sqlite3',
        'ffmpeg-static',
        'ffprobe-static',
        /\.node$/
      ]
    },
    minify: 'esbuild'
  }
})
