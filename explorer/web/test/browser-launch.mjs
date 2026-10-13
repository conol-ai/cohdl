import { existsSync } from 'node:fs'
import { execFileSync } from 'node:child_process'

const candidates = [
  process.env.CHROME_PATH,
  process.env.CHROME,
  process.platform === 'darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : null,
  '/usr/bin/google-chrome',
  '/usr/bin/google-chrome-stable',
  '/usr/bin/chromium',
  '/usr/bin/chromium-browser',
].filter(Boolean)

export function resolveChromeExecutablePath() {
  for (const candidate of candidates) {
    if (existsSync(candidate)) return candidate
  }
  try {
    const resolved = execFileSync('which', ['google-chrome'], { encoding: 'utf8' }).trim()
    if (resolved && existsSync(resolved)) return resolved
  } catch {}
  try {
    const resolved = execFileSync('which', ['chromium'], { encoding: 'utf8' }).trim()
    if (resolved && existsSync(resolved)) return resolved
  } catch {}
  throw new Error('Chrome/Chromium executable not found; set CHROME_PATH in the test environment')
}
