
declare global {
  interface Window {
    appConfig: { host: string, api: number }
    electron: {
      invoke: (channel: string, ...args: any[]) => Promise<any>
      on: (channel: string, callback: (...args: any[]) => void) => void
      off: (channel: string) => void
      send: (channel: string, ...args: any[]) => void
      receive: (channel: string, func: (...args: any[]) => void) => void
      once: (channel: string, func: (...args: any[]) => void) => void
    }
  }
}

export interface ISettings {
  defaultVolume: number
  lastQuery: string | null
  lastFolder: string | string[] | null
  playerCount: number
  layout: 'row' | 'column' | 'grid'
  windowBounds: {
    width: number
    height: number
  }
  windowOpacity: number
  queryStates: Record<string, string>
  mediaFolders: string[]
  language: 'en' | 'es' | 'ja' | 'zh' | 'fr' | 'de' | 'ru' | 'it' | 'pt' | 'ko' | 'hi' | 'bn' | 'ar' | 'id' | 'he' | 'th'
}



export type FormValidation = {
  status?: number
  message?: string
  errors?: { [key: string]: string }
  data?: any
}

export interface IVideoResult {
  success?: boolean
  error?: string
  videos?: {
    path: string
    codec: string
  }[]
}

export interface IVideoList extends Array<string> {
  0: string // video URL
  1: string // codec
  length: 2
}

export interface PlayerMeta {
  id: number
  src: string
  codec: string
  ref?: InstanceType<typeof Player>
  initialTime: number
  initialVolume: number
  initialMuted: boolean
  playHistory: string[]
  forwardHistory: string[]
  playProgress: Map<string, number>
  currentHistoryIndex: number
  playMode: PlayMode
  lastPlayedIndex: number
}

export interface PlayerState {
  videoTime: number
  videoDuration: number
  videoVolume: number
  videoMuted: boolean
  isScrubbing: boolean
  isVolumeScrubbing: boolean
  isOverlayVisible: boolean
  subtitleData: string
  subtitlesEnabled: boolean
  hideVideoBorder: boolean
  lastVideoTime: number
}

export interface VideoEventPayload {
  event: Event
  playerId: number
  videoRef: HTMLVideoElement | null
}

export interface StateChangePayload {
  id: number
  currentTime: number
  duration: number
  volume: number
  muted: boolean
}

export { }
