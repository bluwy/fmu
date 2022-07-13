#[derive(Debug, PartialEq, Eq)]
pub enum JsSyntax {
    ESM,
    CJS,
    Mixed,
    Unknown,
}

// detect file syntax esm or cjs
pub fn get_js_syntax(s: &str) -> JsSyntax {
    let mut i = 0;
    let mut is_esm = false;
    let mut is_cjs = false;
    let b = s.as_bytes();

    while i < b.len() {
        let c = b[i];

        // single line comment, ignore until \n
        if c == b'/' && b[i + 1] == b'/' {
            // TODO: handle \r\n?
            let new_line_pos = match b[i + 2..].iter().position(|&v| v == b'\n') {
                Some(pos) => pos,
                None => break, // assume reach end of file
            };
            i += 2 + new_line_pos + 1;
            continue;
        }

        // multi line comment, ignore until */
        if c == b'/' && b[i + 1] == b'*' {
            let closing_pos = match b[i + 3..]
                .iter()
                .enumerate()
                .position(|(j, &v)| v == b'/' && b[i + 3 + j - 1] == b'*')
            {
                Some(pos) => pos,
                None => break, // assume reach end of file
            };
            i += 3 + closing_pos + 1;
            continue;
        }

        // single and double quotes, ignore until quote end
        if c == b'\'' || c == b'"' {
            let closing_pos = match b[i + 1..].iter().enumerate().position(|(j, &v)| {
                if v == c {
                    // make sure quotes aren't escaped by backtracking the number of backslashes.
                    // we consider unescaped if has 0 or and even number of backslashes.
                    let mut prev_iter_pos = j - 1;
                    while b[i + 1 + prev_iter_pos] == b'\\' {
                        if prev_iter_pos > 0 {
                            prev_iter_pos -= 1;
                        } else {
                            break;
                        }
                    }
                    let backslash_num = j - prev_iter_pos + 1;
                    return backslash_num % 2 == 0;
                } else {
                    return false;
                }
            }) {
                Some(pos) => pos,
                None => break, // assume reach end of file
            };
            i += 1 + closing_pos + 1;
            continue;
        }

        // esm specific detection
        if !is_esm {
            // top-level import
            if c == b'i'
                && b[i + 1] == b'm'
                && b[i + 2] == b'p'
                && b[i + 3] == b'o'
                && b[i + 4] == b'r'
                && b[i + 5] == b't'
                && b[i + 6] == b' '
            {
                is_esm = true;
                i += 7;
                continue;
            }

            // top-level export
            // TODO: ignore variable declaration
            if c == b'e'
                && b[i + 1] == b'x'
                && b[i + 2] == b'p'
                && b[i + 3] == b'o'
                && b[i + 4] == b'r'
                && b[i + 5] == b't'
                && b[i + 6] == b' '
            {
                is_esm = true;
                i += 7;
                continue;
            }
        }

        if !is_cjs {
            // top-level require
            // TODO: skip createRequire
            if c == b'r'
                && b[i + 1] == b'e'
                && b[i + 2] == b'q'
                && b[i + 3] == b'u'
                && b[i + 4] == b'i'
                && b[i + 5] == b'r'
                && b[i + 6] == b'e'
            {
                is_cjs = true;
                i += 7;
                continue;
            }
        }

        i += 1;
    }

    if is_esm && is_cjs {
        return JsSyntax::Mixed;
    } else if is_esm {
        return JsSyntax::ESM;
    } else if is_cjs {
        return JsSyntax::CJS;
    } else {
        return JsSyntax::Unknown;
    }
}
