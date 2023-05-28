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
                  if pos > 0 && b[i + 2 + pos + 1] == b'\n' {
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

