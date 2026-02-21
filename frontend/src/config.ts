interface RuntimeConfig {
  apiUrl?: string
  appDomain?: string
  showBusinessPricing?: string
}

declare global {
  interface Window {
    __RUNTIME_CONFIG__?: RuntimeConfig
  }
}

const rc = window.__RUNTIME_CONFIG__

export const config = {
  apiUrl: rc?.apiUrl || import.meta.env.VITE_API_URL || '',
  appDomain: rc?.appDomain || import.meta.env.VITE_APP_DOMAIN || '',
  showBusinessPricing:
    (rc?.showBusinessPricing || import.meta.env.VITE_SHOW_BUSINESS_PRICING || 'false') === 'true',
}
