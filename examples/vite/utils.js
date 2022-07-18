// Copy from publint to compare perf

// Reference: https://github.com/unjs/mlly/blob/c5ae321725cbabe230c16c315d474c36eee6a30c/src/syntax.ts#L7
const ESM_CONTENT_RE =
  /([\s;]|^)(import[\w,{}\s*]*from|import\s*['"*{]|export\b\s*(?:[*{]|default|type|function|const|var|let|async function)|import\.meta\b)/m
export function isCodeEsm(code) {
  return ESM_CONTENT_RE.test(code)
}

// Reference: https://github.com/unjs/mlly/blob/c5ae321725cbabe230c16c315d474c36eee6a30c/src/syntax.ts#L15
const CJS_CONTENT_RE =
  /([\s;]|^)(module.exports\b|exports\.\w|require\s*\(|global\.\w|Object\.(defineProperty|defineProperties|assign)\s*\(\s*exports\b)/m
export function isCodeCjs(code) {
  return CJS_CONTENT_RE.test(code)
}

const MULTILINE_COMMENTS_RE = /\/\*(.|[\r\n])*?\*\//gm
const SINGLELINE_COMMENTS_RE = /\/\/.*/g
export function stripComments(code) {
  return code
    .replace(MULTILINE_COMMENTS_RE, '')
    .replace(SINGLELINE_COMMENTS_RE, '')
}

/**
 * @param {string} code
 * @returns {CodeFormat}
 */
export function getCodeFormat(code) {
  code = stripComments(code)
  const isEsm = isCodeEsm(code)
  const isCjs = isCodeCjs(code)
  // In reality, a file can't have mixed ESM and CJS. It's syntactically incompatible in both environments.
  // But since we use regex, we can't correct identify ESM and CJS, so when this happens we should bail instead.
  // TODO: Yak shave a correct implementation.
  if (isEsm && isCjs) {
    return 'Mixed'
  } else if (isEsm) {
    return 'ESM'
  } else if (isCjs) {
    return 'CJS'
  } else {
    // If we can't determine the format, it's likely that it doesn't import/export and require/exports.
    // Meaning it's a side-effectful file, which would always match the `format`
    return 'Unknown'
  }
}
