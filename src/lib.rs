// suppress wasm-bindgen auto-generated name warning
#![allow(non_snake_case, non_upper_case_globals)]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, PartialEq, Eq)]
pub enum JsSyntax {
    ESM,
    CJS,
    Mixed,
    Unknown,
}

// detect file syntax esm or cjs
#[wasm_bindgen(js_name = "guessJsSyntax")]
pub fn guess_js_syntax(s: &str) -> JsSyntax {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let mut i = 0;
    let mut is_esm = false;
    let mut is_cjs = false;
    let b = s.as_bytes();

    // parsing state variables
    //
    // template literals can contain js via ${}, when this happens, we increase the depth by 1
    // for each js within. When the depth is more than 0, we need to do special checks to
    // know when we reach the end of the js.
    //
    // const foo = `hello ${world}`
    // |----------||------||-----||
    //       0                1
    let mut template_literal_js_depth = 0;
    // shadowing
    // default depth is 0, every open brace increments, closing brace decrements.
    // this happens for JS objects too but for us, it's good enough
    let mut scope_depth = 0;
    let mut require_shadowed_depth = usize::MAX;
    let mut module_shadowed_depth = usize::MAX;
    let mut exports_shadowed_depth = usize::MAX;

    while i < b.len() && !(is_esm && is_cjs) {
        let c = b[i];

        // single line comment, ignore until \n
        if c == b'/' && b[i + 1] == b'/' {
            let new_line_pos = match b[i + 2..].iter().position(|&v| v == b'\n' || v == b'\r') {
                Some(pos) => {
                  if pos > 0 && b[i + 2 + pos] == b'\r' {
                    pos + 1
                  } else {
                    pos
                  }
                },
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
            // println!("quotes {}", &s[i..i + 1 + closing_pos + 1]);
            i += 1 + closing_pos + 1;
            continue;
        }

        // template literal, skip until ` or ${
        // template literal, but is inner js code, also check for closing }
        if c == b'`' || (template_literal_js_depth > 0 && c == b'}') {
            let closing_pos = match b[i + 1..].iter().enumerate().position(|(j, &v)| {
                // capture ${
                if v == b'$' && b[i + 1 + j + 1] == b'{' {
                    return !is_backslash_escaped(b, i + 1 + j);
                }
                // capture `
                if v == b'`' {
                    return !is_backslash_escaped(b, i + 1 + j);
                }
                false
            }) {
                Some(pos) => pos,
                None => break, // assume reach end of file
            };
            if b[i + 1 + closing_pos] == b'$' {
                // only increment for `, since for ${ it's already incremented
                if c == b'`' {
                    template_literal_js_depth += 1;
                }
                // println!("temlitopen {}", &s[i..i + 1 + closing_pos + 2]);
                i += 1 + closing_pos + 2;
            } else {
                // only decrement for }, since for ` it's already decremented
                if c == b'}' {
                    template_literal_js_depth -= 1;
                }
                // println!("temlitclose {}", &s[i..i + 1 + closing_pos + 1]);
                i += 1 + closing_pos + 1;
            }
            continue;
        };

        // skip regex
        if c == b'/' {
            let left = get_nearest_non_whitespace_index_left(&b, i);
            // knowing when a / is a division or the start of a regex ia PAIN. luckily this below
            // works good enough, by checking the we're not preceding any variables, but if is,
            // only allow specific keywords (see function for specific keywords)
            // Thanks for inspiration: https://github.com/guybedford/es-module-lexer/blob/559a550318fcdfe20c60cb322c147905b5aadf9f/src/lexer.c#L186-L200
            if !b[left].is_ascii_alphanumeric()
                || is_slash_preceded_by_regex_possible_keyword(&b, left)
            {
                // mini [] state, anything in [] is literal, so skip / detection
                let mut is_in_bracket = false;
                let re_closing_pos = match b[i + 1..].iter().enumerate().position(|(j, &v)| {
                    if v == b'[' && !is_backslash_escaped(b, i + 1 + j) {
                        is_in_bracket = true;
                        return false;
                    } else if v == b']' && !is_backslash_escaped(b, i + 1 + j) {
                        is_in_bracket = false;
                        return false;
                    } else if v == b'\n' {
                        // TODO: this might be redundant now that i implemented the divisiion / regex heuristic
                        return true;
                    } else if !is_in_bracket && v == b'/' && !is_backslash_escaped(b, i + 1 + j) {
                        return true;
                    }
                    false
                }) {
                    Some(pos) => pos,
                    None => break, // assume reach end of file
                };
                if b[i + 1 + re_closing_pos] == b'\n' {
                    // it's a division, not a regex
                    i += 1;
                    continue;
                } else {
                    // we also need to skip regex modifiers
                    let re_modifier_pos = match b[i + 1 + re_closing_pos + 1..]
                        .iter()
                        .position(|&v| !v.is_ascii_alphabetic())
                    {
                        Some(pos) => pos,
                        None => break, // assume reach end of file
                    };
                    // println!(
                    //     "regex {}",
                    //     &s[i..i + 1 + re_closing_pos + 1 + re_modifier_pos]
                    // );
                    i += 1 + re_closing_pos + 1 + re_modifier_pos;
                    continue;
                }
            }
        }

        // esm specific detection
        if !is_esm {
            // top-level import
            if is_import_identifier(&b, i) {
                // TODO: handle space between import.meta, but why would someone do that
                if b[i + 1] == b'.' && is_meta_identifier(&b, i + 2) {
                    is_esm = true;
                    i += 11;
                } else {
                    // TODO: handle \r\n?
                    for &v in b[i + 6..].iter() {
                        if v == b'\'' || v == b'"' || v == b'{' || v == b'\n' {
                            is_esm = true;
                            break;
                        } else if v == b'(' {
                            // dynamic import
                            break;
                        }
                    }
                    i += 6;
                }
                continue;
            }

            // top-level export
            if is_export_identifier(&b, i) {
                is_esm = true;
                i += 6;
                continue;
            }
        }

        if !is_cjs {
            // track scope depth
            // NOTE: track in cjs only as it's only relevant for it
            // TODO: track `=>` and `?:` scoped (pita)
            if c == b'{' {
                scope_depth += 1;
            } else if c == b'}' {
                scope_depth -= 1;
                // re-concile shadowed depth, if we exit the scope that has been
                // shadowed by require, module, or exports, reset them
                if scope_depth < require_shadowed_depth {
                    require_shadowed_depth = usize::MAX;
                }
                if scope_depth < module_shadowed_depth {
                    module_shadowed_depth = usize::MAX;
                }
                if scope_depth < exports_shadowed_depth {
                    exports_shadowed_depth = usize::MAX;
                }
            }

            // require reference
            if scope_depth < require_shadowed_depth && is_require_identifier(&b, i) {
                if is_var_declaration(&b, i) {
                    require_shadowed_depth = scope_depth;
                } else if is_function_param_declaration(&b, i, i + 7) {
                    require_shadowed_depth = scope_depth + 1;
                } else {
                    is_cjs = true;
                }
                i += 7;
                continue;
            }

            // module reference
            if scope_depth < module_shadowed_depth && is_module_identifier(&b, i) {
                if is_var_declaration(&b, i) {
                    module_shadowed_depth = scope_depth;
                } else if is_function_param_declaration(&b, i, i + 6) {
                    module_shadowed_depth = scope_depth + 1;
                } else {
                    is_cjs = true;
                }
                i += 6;
                continue;
            }

            // exports reference
            if scope_depth < exports_shadowed_depth && is_exports_identifier(&b, i) {
                if is_var_declaration(&b, i) {
                    exports_shadowed_depth = scope_depth;
                } else if is_function_param_declaration(&b, i, i + 7) {
                    exports_shadowed_depth = scope_depth + 1;
                } else {
                    is_cjs = true;
                }
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

// safe index (shorten to not be annoying)
fn si(i: usize, b: &[u8]) -> usize {
    i.min(b.len() - 1)
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

// make sure the identifier is it itself, e.g. match `import` not `blaimport`.
// this works the same was as regex \b
fn is_word_bounded(
    full_str: &[u8],
    identifier_start_index: usize,
    identifier_end_index: usize,
) -> bool {
    let left_bounded = identifier_start_index <= 0
        || !full_str[identifier_start_index - 1].is_ascii_alphanumeric();
    let right_bounded = identifier_end_index >= full_str.len()
        || !full_str[identifier_end_index].is_ascii_alphanumeric();
    left_bounded && right_bounded
}

// walks to the left until a non-whitespace character is found.
// return 0 if out of string bounds
fn get_nearest_non_whitespace_index_left(full_str: &[u8], char_index: usize) -> usize {
    if char_index <= 0 {
        return 0;
    }
    let mut i = char_index;
    while i > 0 {
        i -= 1;
        if !full_str[i].is_ascii_whitespace() {
            break;
        }
    }
    i
}

// walks to the right until a non-whitespace character is found.
// return string last index if out of string bounds
fn get_nearest_non_whitespace_index_right(full_str: &[u8], char_index: usize) -> usize {
    if char_index >= full_str.len() - 1 {
        return full_str.len() - 1;
    }
    let mut i = char_index;
    loop {
        if !full_str[i].is_ascii_whitespace() {
            break;
        }
        if i < full_str.len() - 1 {
            i += 1;
        } else {
            break;
        }
    }
    i
}

fn is_import_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'i'
        && full_str[si(iter_index + 1, full_str)] == b'm'
        && full_str[si(iter_index + 2, full_str)] == b'p'
        && full_str[si(iter_index + 3, full_str)] == b'o'
        && full_str[si(iter_index + 4, full_str)] == b'r'
        && full_str[si(iter_index + 5, full_str)] == b't'
        && is_word_bounded(&full_str, iter_index, iter_index + 6)
}

fn is_meta_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'm'
        && full_str[si(iter_index + 1, full_str)] == b'e'
        && full_str[si(iter_index + 2, full_str)] == b't'
        && full_str[si(iter_index + 3, full_str)] == b'a'
        && is_word_bounded(&full_str, iter_index, iter_index + 4)
}

fn is_export_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'e'
        && full_str[si(iter_index + 1, full_str)] == b'x'
        && full_str[si(iter_index + 2, full_str)] == b'p'
        && full_str[si(iter_index + 3, full_str)] == b'o'
        && full_str[si(iter_index + 4, full_str)] == b'r'
        && full_str[si(iter_index + 5, full_str)] == b't'
        && is_word_bounded(&full_str, iter_index, iter_index + 6)
}

fn is_require_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'r'
        && full_str[si(iter_index + 1, full_str)] == b'e'
        && full_str[si(iter_index + 2, full_str)] == b'q'
        && full_str[si(iter_index + 3, full_str)] == b'u'
        && full_str[si(iter_index + 4, full_str)] == b'i'
        && full_str[si(iter_index + 5, full_str)] == b'r'
        && full_str[si(iter_index + 6, full_str)] == b'e'
        && is_word_bounded(full_str, iter_index, iter_index + 7)
}

fn is_module_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'm'
        && full_str[si(iter_index + 1, full_str)] == b'o'
        && full_str[si(iter_index + 2, full_str)] == b'd'
        && full_str[si(iter_index + 3, full_str)] == b'u'
        && full_str[si(iter_index + 4, full_str)] == b'l'
        && full_str[si(iter_index + 5, full_str)] == b'e'
        && is_word_bounded(full_str, iter_index, iter_index + 6)
}

fn is_exports_identifier(full_str: &[u8], iter_index: usize) -> bool {
    full_str[iter_index] == b'e'
        && full_str[si(iter_index + 1, full_str)] == b'x'
        && full_str[si(iter_index + 2, full_str)] == b'p'
        && full_str[si(iter_index + 3, full_str)] == b'o'
        && full_str[si(iter_index + 4, full_str)] == b'r'
        && full_str[si(iter_index + 5, full_str)] == b't'
        && full_str[si(iter_index + 6, full_str)] == b's'
        && is_word_bounded(full_str, iter_index, iter_index + 7)
}

// check if preceded by var, let, const
fn is_var_declaration(full_str: &[u8], identifier_start_index: usize) -> bool {
    let prev_non_whitespace_index =
        get_nearest_non_whitespace_index_left(full_str, identifier_start_index);

    // var
    if full_str[prev_non_whitespace_index] == b'r'
        && full_str[prev_non_whitespace_index.saturating_sub(1)] == b'a'
        && full_str[prev_non_whitespace_index.saturating_sub(2)] == b'v'
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
        && full_str[prev_non_whitespace_index.saturating_sub(1)] == b'e'
        && full_str[prev_non_whitespace_index.saturating_sub(2)] == b'l'
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
        && full_str[prev_non_whitespace_index.saturating_sub(1)] == b's'
        && full_str[prev_non_whitespace_index.saturating_sub(2)] == b'n'
        && full_str[prev_non_whitespace_index.saturating_sub(3)] == b'o'
        && full_str[prev_non_whitespace_index.saturating_sub(4)] == b'c'
        && is_word_bounded(
            full_str,
            prev_non_whitespace_index - 4,
            prev_non_whitespace_index + 1,
        )
    {
        return true;
    }

    false
}

// check if it's a function parameter that's creating a scope
fn is_function_param_declaration(
    full_str: &[u8],
    identifier_start_index: usize,
    identifier_end_index: usize,
) -> bool {
    let prev_non_whitespace_index =
        get_nearest_non_whitespace_index_left(full_str, identifier_start_index);
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
    if full_str[next_non_whitespace_index] == b')' || full_str[next_non_whitespace_index] == b',' {
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

// whether the identifier is preceded by a regex-possible keyword (in reverse check)
// if, else, return, while, yield
fn is_slash_preceded_by_regex_possible_keyword(full_str: &[u8], char_index: usize) -> bool {
    if full_str[char_index] == b'f' && full_str[char_index.saturating_sub(1)] == b'i' {
        return true;
    }
    if full_str[char_index] == b'e'
        && full_str[char_index.saturating_sub(1)] == b's'
        && full_str[char_index.saturating_sub(2)] == b'l'
        && full_str[char_index.saturating_sub(3)] == b'e'
    {
        return true;
    }
    if full_str[char_index] == b'n'
        && full_str[char_index.saturating_sub(1)] == b'r'
        && full_str[char_index.saturating_sub(2)] == b'u'
        && full_str[char_index.saturating_sub(3)] == b't'
        && full_str[char_index.saturating_sub(4)] == b'e'
        && full_str[char_index.saturating_sub(5)] == b'r'
    {
        return true;
    }
    if full_str[char_index] == b'e'
        && full_str[char_index.saturating_sub(1)] == b'l'
        && full_str[char_index.saturating_sub(2)] == b'i'
        && full_str[char_index.saturating_sub(3)] == b'h'
        && full_str[char_index.saturating_sub(4)] == b'w'
    {
        return true;
    }
    if full_str[char_index] == b'd'
        && full_str[char_index.saturating_sub(1)] == b'l'
        && full_str[char_index.saturating_sub(2)] == b'e'
        && full_str[char_index.saturating_sub(3)] == b'i'
        && full_str[char_index.saturating_sub(3)] == b'y'
    {
        return true;
    }
    false
}
