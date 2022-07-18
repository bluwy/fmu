import { init, guessJsSyntax } from 'fmu'
import wasmUrl from 'fmu/wasm?url'
// import { getCodeFormat } from './utils'
import './style.css'

const textarea = document.getElementById('js-textarea')
const result = document.getElementById('result')
const timing = document.getElementById('timing')
const metadata = document.getElementById('metadata')

init(wasmUrl).then(() => {
  update()
  // debounce update
  let t
  textarea.addEventListener('input', () => {
    clearTimeout(t)
    t = setTimeout(update, 300)
  })
})

function update() {
  try {
    const start = performance.now()
    result.innerText = guessJsSyntax(textarea.value)
    const end = performance.now()
    timing.innerText = ` (${(end - start).toPrecision(3)}ms)`

    // So turns out that JS regex is faster than Rust wasm but the Rust implementation
    // is currently more correct. If we want to match the correctness in JS, Rust should win.
    // For now let's hide it :P
    //
    // const start2 = performance.now()
    // const format = getCodeFormat(textarea.value)
    // const end2 = performance.now()
    // metadata.innerText = `Regex is ${format} (${(end2 - start2).toPrecision(
    //   3
    // )}ms)`
  } catch (e) {
    result.innerText = '[Error] see console for details'
    timing.innerText = ''
    metadata.innerText = ''
    throw e
  }
}
