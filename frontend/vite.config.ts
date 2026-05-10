import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import { execSync } from 'child_process'
import pkg from './package.json'

function safeGit(args: string): string {
  try {
    return execSync(`git ${args}`, { stdio: ['pipe', 'pipe', 'ignore'] })
      .toString()
      .trim()
  } catch {
    return 'unknown'
  }
}

const APP_VERSION = process.env.APP_VERSION || pkg.version
const GIT_COMMIT = process.env.GIT_COMMIT || safeGit('rev-parse --short HEAD')
const GIT_TAG = process.env.GIT_TAG || safeGit('describe --tags --always --dirty')
const BUILD_DATE = process.env.BUILD_DATE || new Date().toISOString()

export default defineConfig({
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify(APP_VERSION),
    __GIT_COMMIT__: JSON.stringify(GIT_COMMIT),
    __GIT_TAG__: JSON.stringify(GIT_TAG),
    __BUILD_DATE__: JSON.stringify(BUILD_DATE),
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    host: true,
    allowedHosts: ['.a8n.run'],
    proxy: {
      '/api': {
        target: process.env.VITE_API_URL || 'http://localhost:4000',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '/v1'),
      },
    },
    watch: {
      usePolling: true,
    },
  },
})
