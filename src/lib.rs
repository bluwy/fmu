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

    // parse state
    // template literals can contain js via ${}, when this happens, we increase the depth by 1.
    // because this can happen nested, whenever the depth is odd, we are working with js,
    // whenever the depth is even, we are working with template literals.
    // this also affects how we check for closing char.
    let mut template_literal_js_depth = 0;

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
                    return !is_backslash_escaped(b, i + 1 + j);
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

        // template literal
        // or
        // we caught the end of ${}, next up would be resolving the template literal too,
        // so we share this condition
        if c == b'`' || template_literal_js_depth % 2 == 1 && c == b'}' {
            if c == b'}' {
                template_literal_js_depth -= 1;
                // now we're in an even depth (template literal)
            }

            let new_line_pos = match b[i + 1..].iter().enumerate().position(|(j, &v)| {
                // capture ${
                if v == b'$' && b[i + 1 + j + 1] == b'{' {
                    let not_escaped = !is_backslash_escaped(b, i + 1 + j);
                    if not_escaped {
                        template_literal_js_depth += 1;
                    }
                    return not_escaped;
                }
                // capture `
                if v == c {
                    let not_escaped = !is_backslash_escaped(b, i + 1 + j);
                    if not_escaped {
                        // if we're workiing in nested js code, a ` denotes a new nested template literal,
                        // increase depth by 1.
                        if template_literal_js_depth % 2 == 1 {
                            template_literal_js_depth += 1;
                        } else {
                            template_literal_js_depth -= 1;
                        }
                    }
                    return not_escaped;
                } else {
                    return false;
                }
            }) {
                Some(pos) => pos,
                None => break, // assume reach end of file
            };
            i += 1 + new_line_pos + 1;
            continue;
        }

        // skip regex
        if c == b'/' && b[i + 1] != b'/' {
            let re_closing_pos = match b[i + 1..]
                .iter()
                .enumerate()
                .position(|(j, &v)| v == b'/' && !is_backslash_escaped(b, i + 1 + j))
            {
                Some(pos) => pos,
                None => break, // assume reach end of file
            };
            // we also need to skip regex modifiers
            let re_modifier_pos = match b[i + 1 + re_closing_pos + 1..]
                .iter()
                .position(|&v| !v.is_ascii_alphabetic())
            {
                Some(pos) => pos,
                None => break, // assume reach end of file
            };
            i += 1 + re_closing_pos + 1 + re_modifier_pos + 1;
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
                && !b[i + 6].is_ascii_alphabetic()
                && (i == 0 || !b[i - 1].is_ascii_alphabetic())
            {
                // TODO: handle \r\n?
                for &v in b[i + 6..].iter() {
                    if v == b'\'' || v == b'"' || v == b'\n' {
                        is_esm = true;
                        break;
                    } else if v == b'(' {
                        // dynamic import
                        break;
                    }
                }
                i += 7;
                if is_esm {
                    continue;
                }
            }

            // top-level export
            // TODO: ignore variable declaration
            if c == b'e'
                && b[i + 1] == b'x'
                && b[i + 2] == b'p'
                && b[i + 3] == b'o'
                && b[i + 4] == b'r'
                && b[i + 5] == b't'
                && !b[i + 6].is_ascii_alphabetic()
                && (i == 0 || !b[i - 1].is_ascii_alphabetic())
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
                && !b[i + 7].is_ascii_alphabetic()
                && (i == 0 || !b[i - 1].is_ascii_alphabetic())
            {
                println!("{}", "cjs");
                is_cjs = true;
                i += 8;
                continue;
            }

            // module reference
            // TODO: skip scoped variables
            if c == b'm'
                && b[i + 1] == b'o'
                && b[i + 2] == b'd'
                && b[i + 3] == b'u'
                && b[i + 4] == b'l'
                && b[i + 5] == b'e'
                && !b[i + 6].is_ascii_alphabetic()
                && (i == 0 || !b[i - 1].is_ascii_alphabetic())
            {
                is_cjs = true;
                i += 7;
                continue;
            }

            // exports reference
            // TODO: skip scoped variables
            if c == b'e'
                && b[i + 1] == b'x'
                && b[i + 2] == b'p'
                && b[i + 3] == b'o'
                && b[i + 4] == b'r'
                && b[i + 5] == b't'
                && b[i + 6] == b's'
                && !b[i + 7].is_ascii_alphabetic()
                && (i == 0 || !b[i - 1].is_ascii_alphabetic())
            {
                is_cjs = true;
                i += 8;
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

// make sure things aren't escaped by backtracking the number of backslashes.
// we consider escaped if has an odd number of backslashes.
fn is_backslash_escaped(full_str: &[u8], char_index: usize) -> bool {
    let mut prev_iter_pos = char_index - 1;
    while full_str[prev_iter_pos] == b'\\' {
        if prev_iter_pos > 0 {
            prev_iter_pos -= 1;
        } else {
            break;
        }
    }
    let backslash_num = char_index - prev_iter_pos + 1;
    backslash_num % 2 == 1
}
