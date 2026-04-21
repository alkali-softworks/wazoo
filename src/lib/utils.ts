import type { Ref } from 'vue'
import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function log(...args: any[]) {
  const isDev = process.env.NODE_ENV === 'development'
  if (isDev) {
    console.log(...arguments)
  }
}

export const baseUrl = (): string => {
  let port = localStorage.getItem('hono-port')
  if (!port || port === 'null') {
    port = '3000'
  }

  if (process.env.NODE_ENV === 'development') {
    return ``
  }

  return `http://localhost:${port}`
}

export const formatTime = (timeInSeconds: number): string => {
  if (!timeInSeconds) return '00:00'
  const minutes = Math.floor(timeInSeconds / 60)
  const seconds = Math.floor(timeInSeconds % 60)
  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
}

export function cleanName(name: string): string {
  if (!name) return ''

  // Replace dots and underscores with spaces
  let cleaned = name.replace(/[._]/g, ' ')

  // Remove content in square brackets
  cleaned = cleaned.replace(/\[[^\]]*\]/g, '')

  // Keep parentheses ONLY if they contain a 4-digit year (e.g., (1994))
  // Otherwise remove parentheses and their content
  cleaned = cleaned.replace(/\((?!\d{4}\))[^)]*\)/g, '')

  // Remove common metadata tags
  const metadataTags = [
    /dvdrip/i, /bdrip/i, /bluray/i, /webrip/i, /hdrip/i,
    /x264/i, /x265/i, /h264/i, /h265/i, /hevc/i,
    /1080p/i, /720p/i, /480p/i, /2160p/i, /4k/i,
    /aac/i, /dts/i, /ac3/i, /flac/i,
    /complete/i
  ]

  metadataTags.forEach(tag => {
    cleaned = cleaned.replace(tag, '')
  })

  // Clean up multiple spaces
  cleaned = cleaned.replace(/\s+/g, ' ')

  // Remove trailing group name or dash (e.g., -VXT, -HDS)
  cleaned = cleaned.replace(/-\w+$/, '')

  // Final trim and cleanup of trailing punctuation
  return cleaned
    .trim()
    .replace(/[-.\s]+$/, '')
}

export const formatVideoFolder = (url: string): string => {
  // Handle both Windows and Unix-style paths
  const segments = url.split(/[/\\]/)
  let folderName = segments[segments.length - 2] || ''
  return cleanName(folderName)
}

export const formatVideoTitle = (url: string): string => {
  // Handle both Windows and Unix-style paths
  const fileName = url.split(/[/\\]/).pop() || ''

  // Remove file extension
  const withoutExtension = fileName.replace(/\.[^/.]+$/, '')

  // Remove episode numbering patterns
  const withoutEpisodeNum = withoutExtension
    .replace(/(\d+x\d+) - /, '') // Removes "1x21 - " pattern
    .replace(/\(S\d+\)/, '') // Removes "(S01)" pattern

  return cleanName(withoutEpisodeNum)
}

export function debounce<T extends (...args: any[]) => void>(func: T, delay: number) {
  let timeout: ReturnType<typeof setTimeout> | null = null
  return function (this: ThisParameterType<T>, ...args: Parameters<T>) {
    if (timeout) {
      clearTimeout(timeout)
    }
    timeout = setTimeout(() => {
      func.apply(this, args)
    }, delay)
  }
}