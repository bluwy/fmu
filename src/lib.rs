// suppress wasm-bindgen auto-generated name warning
#![allow(non_snake_case, non_upper_case_globals)]

mod utils;
mod walk;

use utils::{
    is_export_identifier, is_exports_identifier, is_function_param_declaration,
    is_import_identifier, is_meta_identifier, is_module_identifier, is_require_identifier,
    is_var_declaration,
};
use walk::{walk, WalkCallbackResult};
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

    let mut is_esm = false;
    let mut is_cjs = false;

    // shadowing
    // default depth is 0, every open brace increments, closing brace decrements.
    // this happens for JS objects too but for us, it's good enough
    let mut scope_depth = 0;
    let mut require_shadowed_depth = usize::MAX;
    let mut module_shadowed_depth = usize::MAX;
    let mut exports_shadowed_depth = usize::MAX;

    walk(s, |b, i, c| {
        if is_esm && is_cjs {
            return WalkCallbackResult::Break;
        }

        // esm specific detection
        if !is_esm {
            // top-level import
            if is_import_identifier(&b, *i) {
                // TODO: handle space between import.meta, but why would someone do that
                if b[*i + 1] == b'.' && is_meta_identifier(&b, *i + 2) {
                    is_esm = true;
                    *i += 11;
                } else {
                    // TODO: handle \r\n?
                    for &v in b[*i + 6..].iter() {
                        if v == b'\'' || v == b'"' || v == b'{' || v == b'\n' {
                            is_esm = true;
                            break;
                        } else if v == b'(' {
                            // dynamic import
                            break;
                        }
                    }
                    *i += 6;
                }
                return WalkCallbackResult::Continue;
            }

            // top-level export
            if is_export_identifier(&b, *i) {
                is_esm = true;
                *i += 6;
                return WalkCallbackResult::Continue;
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
            if scope_depth < require_shadowed_depth && is_require_identifier(&b, *i) {
                if is_var_declaration(&b, *i) {
                    require_shadowed_depth = scope_depth;
                } else if is_function_param_declaration(&b, *i, *i + 7) {
                    require_shadowed_depth = scope_depth + 1;
                } else {
                    is_cjs = true;
                }
                *i += 7;
                return WalkCallbackResult::Continue;
            }

            // module reference
            if scope_depth < module_shadowed_depth && is_module_identifier(&b, *i) {
                if is_var_declaration(&b, *i) {
                    module_shadowed_depth = scope_depth;
                } else if is_function_param_declaration(&b, *i, *i + 6) {
                    module_shadowed_depth = scope_depth + 1;
                } else {
                    is_cjs = true;
                }
                *i += 6;
                return WalkCallbackResult::Continue;
            }

            // exports reference
            if scope_depth < exports_shadowed_depth && is_exports_identifier(&b, *i) {
                if is_var_declaration(&b, *i) {
                    exports_shadowed_depth = scope_depth;
                } else if is_function_param_declaration(&b, *i, *i + 7) {
                    exports_shadowed_depth = scope_depth + 1;
                } else {
                    is_cjs = true;
                }
                *i += 7;
                return WalkCallbackResult::Continue;
            }
        }

        return WalkCallbackResult::Continue;
    });

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
