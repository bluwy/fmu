// In this file, we make it explicitly ESM, but entice it to be CJS.
// Make sure the parser knows this is still ESM.

import foo from 'bar'
export const bar = 'foo'
export { foo }

// require('hello')

/*
  require('world')
*/

export function entice(module, exports) {
  let a = `require('bar')`
  module.exports = { strange: 'package' }
  exports = 'chaos!'
  const regexMadness = /require('asdsd')/gi
}
