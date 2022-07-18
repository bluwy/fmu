import init, {
  initSync,
  guessJsSyntax as _guessJsSyntax
} from '../dist/index.js'

export function guessJsSyntax(s) {
  const result = _guessJsSyntax(s)
  switch (result) {
    case 0:
      return 'ESM'
    case 1:
      return 'CJS'
    case 2:
      return 'Mixed'
    case 3:
      return 'Unknown'
  }
}

export { init, initSync }
