export interface BuildInfo {
  version: string
  gitTag: string
  commit: string
  buildDate: string
}

export const buildInfo: BuildInfo = {
  version: __APP_VERSION__,
  gitTag: __GIT_TAG__,
  commit: __GIT_COMMIT__,
  buildDate: __BUILD_DATE__,
}

declare global {
  interface Window {
    __APP_INFO__?: BuildInfo
  }
}

if (typeof window !== 'undefined') {
  window.__APP_INFO__ = buildInfo
}
