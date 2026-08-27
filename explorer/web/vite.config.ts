import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Dev HMR composes with the extractor's live server: `npm run dev` serves the
// frontend with hot module replacement on 5198 and proxies /api (model, SSE
// events, photos, files) to the extractor on 5199 — run both and edits to
// either the .cohdl source or the frontend reload live. `preview` serves the
// built dist on the extractor's port as a drop-in stand-in, so keep them equal.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5198,
    proxy: {
      '/api': 'http://127.0.0.1:5199',
    },
  },
  preview: { port: 5199 },
})
