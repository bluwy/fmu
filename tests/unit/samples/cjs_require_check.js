let foo

// NOTE: cases like this ideally means that this file is not CJS, as it works in ESM too.
// but it is really really hard to detect unless we do control flow analysis.
if (typeof require !== 'undefined') {
  foo = require('foo')
} else {
  foo = 'bar'
}
