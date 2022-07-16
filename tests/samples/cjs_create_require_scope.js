const { createRequire } = require('module')

function main() {
  const require = createRequire(__filename)
  require('foo')
}

// make sure main's `require` affects it's scope only
require('foo')
main()
