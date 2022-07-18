export type JsSyntax = 'ESM' | 'CJS' | 'Mixed' | 'Unknown'

export function guessJsSyntax(s: string): JsSyntax

export {
  default as init,
  initSync,
  InitInput,
  InitOutput
} from '../dist/index.js'
