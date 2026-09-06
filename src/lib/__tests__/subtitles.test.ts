import { describe, it, expect } from 'vitest'
import { isAssOrSsa, srtToAss, normalizeSubtitles } from '@/lib/subtitles'

describe('subtitles utilities', () => {
  describe('isAssOrSsa', () => {
    it('detects ASS with [Script Info]', () => {
      const ass = `[Script Info]\nTitle: Test\n[Events]\nDialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,Hello`
      expect(isAssOrSsa(ass)).toBe(true)
    })

    it('detects SSA with [V4 Styles]', () => {
      const ssa = `[V4 Styles]\nFormat: Name\n[Events]\nDialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,Hello`
      expect(isAssOrSsa(ssa)).toBe(true)
    })

    it('returns false for SRT', () => {
      const srt = `1\n00:00:01,000 --> 00:00:02,000\nHello world`
      expect(isAssOrSsa(srt)).toBe(false)
    })

    it('returns false for WebVTT', () => {
      const vtt = `WEBVTT\n\n00:00:01.000 --> 00:00:02.000\nHello world`
      expect(isAssOrSsa(vtt)).toBe(false)
    })
  })

  describe('srtToAss', () => {
    it('converts SRT timestamps and multi-line text to ASS', () => {
      const srt = `1
00:01:23,456 --> 00:01:26,789
Hello world!
Second line

2
00:01:30,000 --> 00:01:35,500
<i>Italic text</i> and <b>bold text</b>
`
      const ass = srtToAss(srt)
      expect(ass).toContain('[Script Info]')
      expect(ass).toContain('[V4+ Styles]')
      expect(ass).toContain('Dialogue: 0,0:01:23.45,0:01:26.78,Default,,0,0,0,,Hello world!\\NSecond line')
      expect(ass).toContain('Dialogue: 0,0:01:30.00,0:01:35.50,Default,,0,0,0,,{\\i1}Italic text{\\i0} and {\\b1}bold text{\\b0}')
    })

    it('converts font colors to ASS hex BGR format', () => {
      const srt = `1
00:00:05,000 --> 00:00:10,000
<font color="#ff8800">Orange colored text</font>
`
      const ass = srtToAss(srt)
      // #FF8800 -> R=FF, G=88, B=00 -> ASS: &H0088FF&
      expect(ass).toContain('{\\c&H0088FF&}Orange colored text{\\c}')
    })

    it('converts WebVTT files with headers and 2-component timestamps', () => {
      const vtt = `WEBVTT
NOTE This is a comment

01:20.500 --> 01:25.000
WebVTT subtitle here
`
      const ass = srtToAss(vtt)
      expect(ass).toContain('Dialogue: 0,0:01:20.50,0:01:25.00,Default,,0,0,0,,WebVTT subtitle here')
    })
  })

  describe('normalizeSubtitles', () => {
    it('preserves complete ASS content verbatim', () => {
      const ass = `[Script Info]
Title: Original ASS
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,56,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3.2,1.5,2,60,60,60,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,{\\pos(100,200)}Styled`
      expect(normalizeSubtitles(ass)).toBe(ass)
    })


    it('normalizes SRT to ASS', () => {
      const srt = `1\n00:00:01,000 --> 00:00:02,000\nPlain text`
      const normalized = normalizeSubtitles(srt)
      expect(isAssOrSsa(normalized)).toBe(true)
      expect(normalized).toContain('Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,Plain text')
    })
  })
})
