/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_URL: string
  readonly VITE_APP_URL: string
  readonly VITE_APP_DOMAIN: string
  readonly VITE_SHOW_BUSINESS_PRICING: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare const __APP_VERSION__: string
declare const __GIT_COMMIT__: string
declare const __GIT_TAG__: string
declare const __BUILD_DATE__: string
