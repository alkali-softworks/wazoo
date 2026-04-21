/**
 * IPC Wrapper
 *
 * This module provides a safe wrapper around the `window.electron` API.
 * When running in a normal browser (where `window.electron` is undefined),
 * these functions will do nothing or return a safe default (like `null`).
 * This allows your Vue components to run in both Electron and a browser
 * without crashing.
 */

// Check once if we're in Electron.
const isElectron = !!window.electron;

/**
 * Safely invokes an Electron IPC channel.
 * Talks to IPC.Handle
 * Returns a Promise that resolves with the result.
 * In a browser, it returns a Promise that resolves to `null`.
 */
export const ipcInvoke = <T = any>(
  channel: string,
  ...args: any[]
): Promise<T | null> => {
  if (isElectron) {
    return window.electron.invoke(channel, ...args);
  }
  // Log a warning for invokes, as they expect a response
  console.warn(
    `[IPC Wrapper] Tried to 'invoke' channel '${channel}' in a browser. Returning null.`,
  );
  return Promise.resolve(null);
};

/**
 * Safely sends a one-way message to an Electron IPC channel.
 * Talks to IPC.On and IPC.Once
 * In a browser, this does nothing.
 */
export const ipcSend = (channel: string, ...args: any[]): void => {
  if (isElectron) {
    window.electron.send(channel, ...args);
  } else {
    // We don't log 'send' calls by default, as they can be noisy.
    // console.log(`[IPC Wrapper] 'send' to '${channel}' ignored in browser.`);
  }
};

/**
 * Safely registers a listener for an Electron IPC channel.
 * In a browser, this does nothing.
 */
export const ipcReceive = (
  channel: string,
  func: (...args: any[]) => void,
): void => {
  if (isElectron) {
    window.electron.receive(channel, func);
  }
};

/**
 * Safely registers a one-time listener for an Electron IPC channel.
 * In a browser, this does nothing.
 */
export const ipcOnce = (
  channel: string,
  func: (...args: any[]) => void,
): void => {
  if (isElectron) {
    window.electron.once(channel, func);
  }
};

/**
 * A simple boolean to check if the app is running in Electron.
 */
export const isRunningInElectron = isElectron;

// Default export for convenience
export default {
  invoke: ipcInvoke,
  send: ipcSend,
  receive: ipcReceive,
  once: ipcOnce,
  isElectron: isRunningInElectron,
};
