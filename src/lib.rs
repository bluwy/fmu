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
    // shadowing
    let scope_depth = 0;
    let mut require_shadowed_depth = usize::MAX;
    let mut module_shadowed_depth = usize::MAX;
    let mut exports_shadowed_depth = usize::MAX;

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
            if is_import_identifier(&b, i) {
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
            if is_export_identifier(&b, i) {
                is_esm = true;
                i += 7;
                continue;
            }
        }

        if !is_cjs {
            // TODO: ignore variable declaration
            // we're making a quick parse so we can assume every "{" creates a scope, "}" closes a scope.
            // this is important because if a variable called `require`, `module`, `exports` is declared
            // within a scope, further detection should be skip until the scope is closed.
            // so our goal here is to detect those variable declarations, which is not easy.
            // ideally there are only these cases:
            // 1. `var`, `let`, `const` - make sure only capture lhs (ignore multi-declaration for now)
            // 2. `function(){}` - walk forwards and backwards for enclosing parentheses,
            //     and make sure it's followed by a {
            // 3. `() => {}`- use same heuristic as above, but allow =>

            // top-level require
            if require_shadowed_depth < scope_depth && is_require_identifier(&b, i) {
                if is_var_declaration(&b, i, i + 7) {
                    require_shadowed_depth = scope_depth;
                } else {
                    is_cjs = true;
                }
                i += 8;
                continue;
            }

            // module reference
            if module_shadowed_depth < scope_depth && is_module_identifier(&b, i) {
                if is_var_declaration(&b, i, i + 6) {
                    module_shadowed_depth = scope_depth;
                } else {
                    is_cjs = true;
                }
                i += 7;
                continue;
            }

            // exports reference
            if exports_shadowed_depth < scope_depth && is_exports_identifier(&b, i) {
                if is_var_declaration(&b, i, i + 7) {
                    exports_shadowed_depth = scope_depth;
                } else {
                    is_cjs = true;
                }
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

fn is_word_bounded(
    full_str: &[u8],
    identifier_start_index: usize,
    identifier_end_index: usize,
) -> bool {
    !full_str[identifier_end_index].is_ascii_alphabetic()
        && (identifier_start_index == 0
            || !full_str[identifier_start_index - 1].is_ascii_alphabetic())
}

fn get_nearest_non_whitespace_index_left(full_str: &[u8], char_index: usize) -> usize {
    let mut i = char_index;
    while i > 0 && full_str[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    i
}

fn get_nearest_non_whitespace_index_right(full_str: &[u8], char_index: usize) -> usize {
    let mut i = char_index;
    while i < full_str.len() && full_str[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn is_import_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'i'
        && full_str[iter_index + 1] == b'm'
        && full_str[iter_index + 2] == b'p'
        && full_str[iter_index + 3] == b'o'
        && full_str[iter_index + 4] == b'r'
        && full_str[iter_index + 5] == b't'
        && is_word_bounded(&full_str, iter_index, iter_index + 6)
}

fn is_export_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'e'
        && full_str[iter_index + 1] == b'x'
        && full_str[iter_index + 2] == b'p'
        && full_str[iter_index + 3] == b'o'
        && full_str[iter_index + 4] == b'r'
        && full_str[iter_index + 5] == b't'
        && is_word_bounded(&full_str, iter_index, iter_index + 6)
}

fn is_require_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'r'
        && full_str[iter_index + 1] == b'e'
        && full_str[iter_index + 2] == b'q'
        && full_str[iter_index + 3] == b'u'
        && full_str[iter_index + 4] == b'i'
        && full_str[iter_index + 5] == b'r'
        && full_str[iter_index + 6] == b'e'
        && is_word_bounded(full_str, iter_index, iter_index + 7)
}

fn is_module_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'm'
        && full_str[iter_index + 1] == b'o'
        && full_str[iter_index + 2] == b'd'
        && full_str[iter_index + 3] == b'u'
        && full_str[iter_index + 4] == b'l'
        && full_str[iter_index + 5] == b'e'
        && is_word_bounded(full_str, iter_index, iter_index + 6)
}

fn is_exports_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'e'
        && full_str[iter_index + 1] == b'x'
        && full_str[iter_index + 2] == b'p'
        && full_str[iter_index + 3] == b'o'
        && full_str[iter_index + 4] == b'r'
        && full_str[iter_index + 5] == b't'
        && full_str[iter_index + 6] == b's'
        && is_word_bounded(full_str, iter_index, iter_index + 7)
}

// whether the identifier is a variable that
fn is_var_declaration(
    full_str: &[u8],
    identifier_start_index: usize,
    identifier_end_index: usize,
) -> bool {
    // check if preceded by var, let, const
    let prev_non_whitespace_index =
        get_nearest_non_whitespace_index_left(full_str, identifier_start_index);

    // var
    if full_str[prev_non_whitespace_index] == b'r'
        && full_str[prev_non_whitespace_index - 1] == b'a'
        && full_str[prev_non_whitespace_index - 2] == b'v'
        && is_word_bounded(
            full_str,
            prev_non_whitespace_index - 2,
            prev_non_whitespace_index + 1,
        )
    {
        return true;
    }

    // let
    if full_str[prev_non_whitespace_index] == b't'
        && full_str[prev_non_whitespace_index - 1] == b'e'
        && full_str[prev_non_whitespace_index - 2] == b'l'
        && is_word_bounded(
            full_str,
            prev_non_whitespace_index - 2,
            prev_non_whitespace_index + 1,
        )
    {
        return true;
    }

    // const
    if full_str[prev_non_whitespace_index] == b't'
        && full_str[prev_non_whitespace_index - 1] == b's'
        && full_str[prev_non_whitespace_index - 2] == b'n'
        && full_str[prev_non_whitespace_index - 3] == b'o'
        && full_str[prev_non_whitespace_index - 4] == b'c'
        && is_word_bounded(
            full_str,
            prev_non_whitespace_index - 4,
            prev_non_whitespace_index + 1,
        )
    {
        return true;
    }

    let next_non_whitespace_index =
        get_nearest_non_whitespace_index_right(full_str, identifier_end_index);

    // function(identifier) {}
    // function(foo, identifier) {}
    // function(foo, identifier, bar) {}
    // function(foo = identifier) {}
    if full_str[prev_non_whitespace_index] == b'=' {
        return false;
    }
    if full_str[prev_non_whitespace_index] == b'(' || full_str[prev_non_whitespace_index] == b',' {
        return true;
    }
    if full_str[next_non_whitespace_index] == b'(' || full_str[next_non_whitespace_index] == b',' {
        return true;
    }

    // identifier => {}
    if full_str[next_non_whitespace_index] == b'='
        && full_str[next_non_whitespace_index + 1] == b'>'
    {
        return true;
    }

    false
}
