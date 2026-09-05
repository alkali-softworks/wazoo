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
    lib: {
      entry: path.resolve(__dirname, 'src/main/preload.ts'),
      formats: ['cjs']
    },
    rollupOptions: {
      onwarn(warning, warn) {
        if (
          warning.code === 'INVALID_ANNOTATION' ||
          warning.code === 'INEFFECTIVE_DYNAMIC_IMPORT' ||
          warning.code === 'MISSING_NAME_OPTION_FOR_IIFE_EXPORT' ||
          warning.message?.includes('#__PURE__') ||
          warning.message?.includes('inlineDynamicImports')
        ) {
          return
        }
        warn(warning)
      },
      external: [
        'electron'
      ],
      output: {
        entryFileNames: 'preload.js',
        inlineDynamicImports: true
      }
    }
  }
})
