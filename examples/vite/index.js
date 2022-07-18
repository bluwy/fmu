import init, { guessJsSyntax } from 'fmu'
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
  result.innerText = guessJsSyntax(textarea.value)
}
