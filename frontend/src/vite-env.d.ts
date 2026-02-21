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
