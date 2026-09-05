import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import fs from 'fs'
import path from 'path'
import os from 'os'

describe('Subtitle discovery and reading logic', () => {
  let tempDir: string

  beforeAll(async () => {
    tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), 'wazoo-sub-test-'))
  })

  afterAll(async () => {
    await fs.promises.rm(tempDir, { recursive: true, force: true })
  })

  it('prefers .ass over .srt when both exist', async () => {
    const videoBase = 'my-episode-01'
    const assPath = path.join(tempDir, `${videoBase}.ass`)
    const srtPath = path.join(tempDir, `${videoBase}.srt`)

    await fs.promises.writeFile(assPath, '[Script Info]\nTitle: Test ASS', 'utf-8')
    await fs.promises.writeFile(srtPath, '1\n00:00:01,000 --> 00:00:02,000\nTest SRT', 'utf-8')

    const extensions = ['.ass', '.ssa', '.srt', '.vtt']
    let foundSub: string | null = null

    for (const ext of extensions) {
      const candidate = path.join(tempDir, `${videoBase}${ext}`)
      try {
        await fs.promises.access(candidate, fs.constants.F_OK)
        foundSub = candidate
        break
      } catch {
        // continue
      }
    }

    expect(foundSub).toBe(assPath)

    const content = await fs.promises.readFile(foundSub!, 'utf-8')
    expect(content).toContain('Title: Test ASS')
  })

  it('finds language-tagged subtitle files when direct match is absent', async () => {
    const videoBase = 'anime-movie'
    const langAssPath = path.join(tempDir, `${videoBase}.en.ass`)

    await fs.promises.writeFile(langAssPath, '[Script Info]\nTitle: English ASS', 'utf-8')

    const extensions = ['.ass', '.ssa', '.srt', '.vtt']
    const files = await fs.promises.readdir(tempDir)
    let matched: string | null = null

    for (const ext of extensions) {
      const match = files.find(f => f.startsWith(`${videoBase}.`) && f.endsWith(ext))
      if (match) {
        matched = path.join(tempDir, match)
        break
      }
    }

    expect(matched).toBe(langAssPath)
  })
})
