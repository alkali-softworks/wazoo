import { defineConfig } from 'vite'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src')
    }
  },
  build: {
    ssr: true,
    target: 'node20',
    outDir: '.vite/build',
    emptyOutDir: false,
    chunkSizeWarningLimit: 3000,
    lib: {
      entry: {
        main: path.resolve(__dirname, 'src/electron/main.ts'),
        scanner: path.resolve(__dirname, 'src/electron/scanner.ts')
      },
      formats: ['cjs']
    },
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
      },
      external: [
        'electron',
        'better-sqlite3',
        'ffmpeg-static',
        'ffprobe-static',
        'bufferutil',
        'utf-8-validate',
        /\.node$/
      ],
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name]-[hash].js',
        assetFileNames: '[name]-[hash][extname]'
      }
    }
  }
})
