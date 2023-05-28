use crate::utils::{
    get_nearest_non_whitespace_index_left, is_backslash_escaped,
    is_slash_preceded_by_regex_possible_keyword,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalkCallbackResult {
    Continue,
    Break,
}

pub fn walk<F>(s: &str, mut cb: F)
where
    F: FnMut(&[u8], &mut usize, u8) -> WalkCallbackResult,
{
    let mut i = 0;
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

    while i < b.len() {
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
                }
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

        let result = cb(b, &mut i, c);
        if result == WalkCallbackResult::Break {
            break;
        } else if result == WalkCallbackResult::Continue {
            i += 1;
        }
    }
}
