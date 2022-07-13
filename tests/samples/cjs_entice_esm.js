// In this file, we make it explicitly CJS, but entice it to be ESM.
// Make sure the parser knows this is still CJS.

// import foo from 'bar'
// export const bar = 'foo'
// export { foo }

import('arrgh')

// import('hello')

/*
  require('world')
*/

require('fs')

module.exports = function entice() {
  let a = `import foo from 'bar'`
}

exports = 'wowzah'

const regexMadness = /import foo from 'bar'/gi
