<br>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://user-images.githubusercontent.com/34116392/179419080-7ef828f6-6f83-429c-ae4b-3d8e40976c4d.svg" height="140">
    <img src="https://user-images.githubusercontent.com/34116392/179418976-8154b41b-087a-4a5a-9254-639707e48a4e.svg" height="140">
  </picture>
</p>

<p align="center">
  A collection of JS module utilities written in Rust
</p>

## Usage

The package is published to npm as a WebAssembly module. Install with:

```bash
$ npm install fmu
```

```js
import { init, guessJsSyntax } from 'fmu'

// initialize wasm (MUST call this before any other APIs)
await init()

const code = `exports.foo = 'bar'`
console.log(await guessJsSyntax(code)) // "CJS"
```

## Development

Follow the [official guide](https://www.rust-lang.org/tools/install) to install Rust.

```bash
# Build wasm for dev (e.g. testing examples)
$ npm run dev

# Build wasm for publishing
$ npm run build

# Run unit and integration tests
$ cargo test
```

## Sponsors

<p align="center">
  <a href="https://bjornlu.com/sponsors.svg">
    <img src="https://bjornlu.com/sponsors.svg" alt="Sponsors" />
  </a>
</p>

## License

MIT
