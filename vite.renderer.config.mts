import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  base: './',
  plugins: [vue()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  build: {
    outDir: '.vite/renderer/main_window',
    emptyOutDir: true,
    cssMinify: 'esbuild',
    chunkSizeWarningLimit: 3000,
    rollupOptions: {
      onwarn(warning, warn) {
        if (
          warning.code === 'INVALID_ANNOTATION' ||
          warning.code === 'INEFFECTIVE_DYNAMIC_IMPORT' ||
          warning.code === 'MISSING_NAME_OPTION_FOR_IIFE_EXPORT' ||
          warning.message?.includes('#__PURE__')
        ) {
          return
        }
        warn(warning)
      }
    }
  },
  optimizeDeps: {
    entries: ['index.html']
  }
})
