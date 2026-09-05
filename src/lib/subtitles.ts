/**
 * Subtitle parsing and conversion utilities.
 * Handles detection and normalization of ASS, SSA, SRT, and WebVTT formats.
 */

/**
 * Checks if the given subtitle text is in ASS or SSA format.
 */
export function isAssOrSsa(content: string): boolean {
  if (!content || typeof content !== 'string') return false
  const clean = content.replace(/^\uFEFF/, '').trim()
  return (
    clean.includes('[Script Info]') ||
    clean.includes('[V4+ Styles]') ||
    clean.includes('[V4 Styles]') ||
    clean.includes('[Events]') ||
    /^\s*Dialogue:\s*\d+/m.test(clean)
  )
}

/**
 * Formats seconds or time string components into ASS timestamp format: H:MM:SS.cc
 */
function formatAssTime(timeStr: string): string {
  const parts = timeStr.trim().replace(',', '.').split(':')
  let hours = 0
  let minutes = 0
  let secondsWithMs = '00.00'

  if (parts.length === 3) {
    hours = parseInt(parts[0], 10) || 0
    minutes = parseInt(parts[1], 10) || 0
    secondsWithMs = parts[2]
  } else if (parts.length === 2) {
    minutes = parseInt(parts[0], 10) || 0
    secondsWithMs = parts[1]
  } else if (parts.length === 1) {
    secondsWithMs = parts[0]
  }

  const secParts = secondsWithMs.split('.')
  const sec = parseInt(secParts[0], 10) || 0
  const msStr = (secParts[1] || '0').padEnd(3, '0').slice(0, 3)
  const centiseconds = Math.floor(parseInt(msStr, 10) / 10)

  const h = hours.toString()
  const m = minutes.toString().padStart(2, '0')
  const s = sec.toString().padStart(2, '0')
  const cs = centiseconds.toString().padStart(2, '0')

  return `${h}:${m}:${s}.${cs}`
}

/**
 * Converts standard HTML tags to ASS override codes.
 */
function htmlToAssTags(text: string): string {
  return text
    .replace(/<[iI]>/g, '{\\i1}')
    .replace(/<\/[iI]>/g, '{\\i0}')
    .replace(/<[bB]>/g, '{\\b1}')
    .replace(/<\/[bB]>/g, '{\\b0}')
    .replace(/<[uU]>/g, '{\\u1}')
    .replace(/<\/[uU]>/g, '{\\u0}')
    .replace(/<[sS]>/g, '{\\s1}')
    .replace(/<\/[sS]>/g, '{\\s0}')
    .replace(/<font\s+color=['"]?#?([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})['"]?>/gi, (_, r, g, b) => {
      // ASS color format is &HBBGGRR&
      return `{\\c&H${b.toUpperCase()}${g.toUpperCase()}${r.toUpperCase()}&}`
    })
    .replace(/<\/font>/gi, '{\\c}')
    .replace(/<[^>]+>/g, '') // Strip any other remaining tags
}

/**
 * Converts SRT or WebVTT subtitles to an ASS script with modern typography and outlines.
 */
export function srtToAss(content: string): string {
  if (!content || typeof content !== 'string') return ''

  const clean = content.replace(/^\uFEFF/, '').replace(/\r\n/g, '\n').replace(/\r/g, '\n')
  const lines = clean.split('\n')

  const timeRegex = /(\d{1,2}:\d{2}(?::\d{2})?(?:[.,]\d{2,3})?)\s*-->\s*(\d{1,2}:\d{2}(?::\d{2})?(?:[.,]\d{2,3})?)/

  const dialogues: string[] = []
  let i = 0

  while (i < lines.length) {
    const line = lines[i].trim()

    // Skip empty lines, WebVTT headers, or NOTE blocks
    if (!line || line.startsWith('WEBVTT') || line.startsWith('NOTE')) {
      i++
      continue
    }

    const match = line.match(timeRegex)
    if (match) {
      const start = formatAssTime(match[1])
      const end = formatAssTime(match[2])
      i++

      const textLines: string[] = []
      while (i < lines.length && lines[i].trim().length > 0) {
        textLines.push(lines[i].trim())
        i++
      }

      const rawText = textLines.join('\\N')
      const formattedText = htmlToAssTags(rawText)

      if (formattedText) {
        dialogues.push(`Dialogue: 0,${start},${end},Default,,0,0,0,,${formattedText}`)
      }
    } else {
      i++
    }
  }

  const header = `[Script Info]
Title: Subtitles
ScriptType: v4.00+
WrapStyle: 0
PlayResX: 1920
PlayResY: 1080
ScaledBorderAndShadow: yes

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,56,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3.2,1.5,2,60,60,60,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
`

  return header + dialogues.join('\n') + '\n'
}

/**
 * Adjusts bottom-aligned styles in ASS/SSA scripts to ensure a comfortable lower buffer,
 * preventing subtitles from being covered by playback controls or progress bars.
 */
export function ensureBottomBuffer(assText: string, minMarginPercent = 0.055): string {
  let playResY = 1080
  const resMatch = assText.match(/PlayResY\s*:\s*(\d+)/i)
  if (resMatch) {
    playResY = parseInt(resMatch[1], 10) || 1080
  }
  const minMargin = Math.round(playResY * minMarginPercent)

  const lines = assText.split('\n')
  let inStyles = false
  let formatFields: string[] = []

  return lines.map(line => {
    const trimmed = line.trim()
    if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
      inStyles = trimmed === '[V4+ Styles]' || trimmed === '[V4 Styles]'
      formatFields = []
      return line
    }
    if (inStyles) {
      if (trimmed.startsWith('Format:')) {
        formatFields = trimmed.slice(7).split(',').map(s => s.trim().toLowerCase())
        return line
      }
      if (trimmed.startsWith('Style:') && formatFields.length > 0) {
        const colonIdx = line.indexOf(':')
        const prefix = line.slice(0, colonIdx + 1)
        const styleContent = line.slice(colonIdx + 1)
        const parts = styleContent.split(',')
        const alignIdx = formatFields.indexOf('alignment')
        const marginVIdx = formatFields.indexOf('marginv')

        if (alignIdx !== -1 && marginVIdx !== -1 && parts.length === formatFields.length) {
          const align = parts[alignIdx].trim()
          // In ASS (V4+), alignments 1, 2, 3 are bottom-aligned.
          // In SSA (V4), alignments 1, 2, 3 are also bottom-aligned.
          if (['1', '2', '3'].includes(align) || !align) {
            const currentMarginV = parseInt(parts[marginVIdx].trim(), 10) || 0
            if (currentMarginV < minMargin) {
              parts[marginVIdx] = ` ${minMargin}`
              return prefix + parts.join(',')
            }
          }
        }
      }
    }
    return line
  }).join('\n')
}

/**
 * Normalizes subtitle content to ASS format regardless of input (ASS, SSA, SRT, WebVTT).
 */
export function normalizeSubtitles(content: string, options?: { minBottomMarginPercent?: number }): string {
  if (!content || typeof content !== 'string') return ''

  let clean = content.replace(/^\uFEFF/, '').replace(/\r\n/g, '\n').replace(/\r/g, '\n').trim()

  if (isAssOrSsa(clean)) {
    // Ensure [Script Info] exists
    if (!clean.includes('[Script Info]')) {
      clean = `[Script Info]\nTitle: Subtitles\nScriptType: v4.00+\nPlayResX: 1920\nPlayResY: 1080\n\n` + clean
    }

    // Ensure [V4+ Styles] or [V4 Styles] exists
    if (!clean.includes('[V4+ Styles]') && !clean.includes('[V4 Styles]')) {
      const defaultStyles = `[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,56,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3.2,1.5,2,60,60,60,1

`
      if (clean.includes('[Events]')) {
        clean = clean.replace('[Events]', defaultStyles + '[Events]')
      } else {
        clean = clean + '\n\n' + defaultStyles
      }
    }

    // Ensure [Events] has a Format line if missing
    if (clean.includes('[Events]')) {
      const eventsSection = clean.slice(clean.indexOf('[Events]'))
      if (!eventsSection.includes('Format:')) {
        clean = clean.replace('[Events]', `[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text`)
      }
    }

    const minMarginPercent = options?.minBottomMarginPercent ?? 0.055
    return ensureBottomBuffer(clean, minMarginPercent)
  }

  return srtToAss(clean)
}

