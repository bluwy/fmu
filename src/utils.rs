// safe index (shorten to not be annoying)
pub fn si(i: usize, b: &[u8]) -> usize {
  i.min(b.len() - 1)
}

// make sure things aren't escaped by backtracking the number of backslashes.
// we consider escaped if has an odd number of backslashes.
pub fn is_backslash_escaped(full_str: &[u8], char_index: usize) -> bool {
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
pub fn is_word_bounded(
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
pub fn get_nearest_non_whitespace_index_left(full_str: &[u8], char_index: usize) -> usize {
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
pub fn get_nearest_non_whitespace_index_right(full_str: &[u8], char_index: usize) -> usize {
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

pub fn is_import_identifier(full_str: &[u8], iter_index: usize) -> bool {
  full_str[iter_index] == b'i'
      && full_str[si(iter_index + 1, full_str)] == b'm'
      && full_str[si(iter_index + 2, full_str)] == b'p'
      && full_str[si(iter_index + 3, full_str)] == b'o'
      && full_str[si(iter_index + 4, full_str)] == b'r'
      && full_str[si(iter_index + 5, full_str)] == b't'
      && is_word_bounded(&full_str, iter_index, iter_index + 6)
}

pub fn is_meta_identifier(full_str: &[u8], iter_index: usize) -> bool {
  full_str[iter_index] == b'm'
      && full_str[si(iter_index + 1, full_str)] == b'e'
      && full_str[si(iter_index + 2, full_str)] == b't'
      && full_str[si(iter_index + 3, full_str)] == b'a'
      && is_word_bounded(&full_str, iter_index, iter_index + 4)
}

pub fn is_export_identifier(full_str: &[u8], iter_index: usize) -> bool {
  full_str[iter_index] == b'e'
      && full_str[si(iter_index + 1, full_str)] == b'x'
      && full_str[si(iter_index + 2, full_str)] == b'p'
      && full_str[si(iter_index + 3, full_str)] == b'o'
      && full_str[si(iter_index + 4, full_str)] == b'r'
      && full_str[si(iter_index + 5, full_str)] == b't'
      && is_word_bounded(&full_str, iter_index, iter_index + 6)
}

pub fn is_require_identifier(full_str: &[u8], iter_index: usize) -> bool {
  full_str[iter_index] == b'r'
      && full_str[si(iter_index + 1, full_str)] == b'e'
      && full_str[si(iter_index + 2, full_str)] == b'q'
      && full_str[si(iter_index + 3, full_str)] == b'u'
      && full_str[si(iter_index + 4, full_str)] == b'i'
      && full_str[si(iter_index + 5, full_str)] == b'r'
      && full_str[si(iter_index + 6, full_str)] == b'e'
      && is_word_bounded(full_str, iter_index, iter_index + 7)
}

pub fn is_module_identifier(full_str: &[u8], iter_index: usize) -> bool {
  full_str[iter_index] == b'm'
      && full_str[si(iter_index + 1, full_str)] == b'o'
      && full_str[si(iter_index + 2, full_str)] == b'd'
      && full_str[si(iter_index + 3, full_str)] == b'u'
      && full_str[si(iter_index + 4, full_str)] == b'l'
      && full_str[si(iter_index + 5, full_str)] == b'e'
      && is_word_bounded(full_str, iter_index, iter_index + 6)
}

pub fn is_exports_identifier(full_str: &[u8], iter_index: usize) -> bool {
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
pub fn is_var_declaration(full_str: &[u8], identifier_start_index: usize) -> bool {
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
pub fn is_function_param_declaration(
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
pub fn is_slash_preceded_by_regex_possible_keyword(full_str: &[u8], char_index: usize) -> bool {
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
