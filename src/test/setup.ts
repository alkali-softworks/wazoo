/**
 * Vitest setup file for jsdom environment.
 */
if (typeof globalThis.TextMetrics === 'undefined') {
  globalThis.TextMetrics = class TextMetrics {} as any
}

if (typeof (globalThis as any).CSS === 'undefined') {
  ;(globalThis as any).CSS = {}
}

if (typeof (window as any).CSS === 'undefined') {
  ;(window as any).CSS = {}
}

if (typeof globalThis.ResizeObserver === 'undefined') {
  globalThis.ResizeObserver = class ResizeObserver {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as any
}
