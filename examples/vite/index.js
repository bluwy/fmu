import { init, guessJsSyntax } from 'fmu'
import wasmUrl from 'fmu/wasm?url'
import './style.css'

const textarea = document.getElementById('js-textarea')
const result = document.getElementById('result')

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
    result.innerText = guessJsSyntax(textarea.value)
  } catch (e) {
    result.innerText = '[Error] see console for details'
    throw e
  }
}
