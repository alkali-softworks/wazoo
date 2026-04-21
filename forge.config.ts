import type { ForgeConfig } from '@electron-forge/shared-types'
import { MakerSquirrel } from '@electron-forge/maker-squirrel'
import { MakerZIP } from '@electron-forge/maker-zip'
import { MakerDeb } from '@electron-forge/maker-deb'
import { MakerRpm } from '@electron-forge/maker-rpm'
import { VitePlugin } from '@electron-forge/plugin-vite'
import { FusesPlugin } from '@electron-forge/plugin-fuses'
import { FuseV1Options, FuseVersion } from '@electron/fuses'
import * as path from 'path'
import { cp, mkdir } from 'fs/promises'


// 1. Detect if we are building for Windows (either running ON Windows, or passed via CLI flag)
const isWin = process.platform === 'win32' || process.argv.some(arg => arg.includes('win32'));

const config: ForgeConfig = {
  packagerConfig: {
    icon: './public/icon.png',
    asar: {
      unpack: '{**/node_modules/ffmpeg-static/**/*,**/node_modules/ffprobe-static/**/*}'
    },
    extraResource: [
      'default.db'
    ]
  },
  rebuildConfig: {
    force: true,
    onlyModules: ['better-sqlite3']
  },
  makers: [
    new MakerSquirrel({
      setupIcon: './public/icon.ico'
    }),
    new MakerZIP({}, ['darwin']),
    new MakerRpm({}),
    new MakerDeb({})
  ],
  plugins: [
    new VitePlugin({
      build: [
        {
          entry: 'src/main/main.ts',
          config: 'vite.main.config.ts',
          target: 'main'
        },
        {
          entry: 'src/main/preload.ts',
          config: 'vite.main.config.ts',
          target: 'preload'
        },
        {
          entry: 'src/main/scanner.ts',
          config: 'vite.main.config.ts'
        }
      ],
      renderer: [
        {
          name: 'main_window',
          config: 'vite.renderer.config.ts'
        }
      ]
    }),
    new FusesPlugin({
      version: FuseVersion.V1,
      [FuseV1Options.RunAsNode]: false,
      [FuseV1Options.EnableCookieEncryption]: true,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true,
      [FuseV1Options.OnlyLoadAppFromAsar]: true
    })
  ],

  hooks: {
    async packageAfterCopy(_forgeConfig, buildPath) {
      const requiredNativePackages = [
        'better-sqlite3',
        'bindings',
        'file-uri-to-path',
        'ws',
        'ffmpeg-static',
        'ffprobe-static'
      ]

      const sourceNodeModulesPath = path.resolve(__dirname, 'node_modules')
      const destNodeModulesPath = path.resolve(buildPath, 'node_modules')

      await Promise.all(
        requiredNativePackages.map(async (packageName) => {
          const sourcePath = path.join(sourceNodeModulesPath, packageName)
          const destPath = path.join(destNodeModulesPath, packageName)

          // Adding a quick try/catch here is good practice so future missing 
          // packages warn you instead of silently nuking the build process
          try {
            await mkdir(path.dirname(destPath), { recursive: true })
            await cp(sourcePath, destPath, {
              recursive: true,
              preserveTimestamps: true
            })
          } catch (err: any) {
            if (err.code === 'ENOENT') {
              console.warn(`\n⚠️ Warning: Native package '${packageName}' not found. Skipping copy.`)
            } else {
              throw err;
            }
          }
        })
      )
    }
  }
}

export default config
