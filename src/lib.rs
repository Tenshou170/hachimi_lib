use std::borrow::Cow;

use textwrap::{core::Word, wrap_algorithms, WordSeparator::UnicodeBreakProperties};
use wasm_bindgen::prelude::*;

pub struct IsolateTags<'a> {
    s: &'a str,
    bytes: std::str::Bytes<'a>,
    i: usize,
    current_byte: Option<u8>,
}

impl<'a> IsolateTags<'a> {
    pub fn new(s: &'a str) -> Self {
        let mut bytes = s.bytes();
        Self {
            current_byte: bytes.next(),
            s,
            bytes,
            i: 0,
        }
    }
}

impl<'a> Iterator for IsolateTags<'a> {
    type Item = (&'a str, bool);

    fn next(&mut self) -> Option<Self::Item> {
        self.current_byte?;

        let start = self.i;
        // Unity tags
        let mut tag_start = 0;
        let mut in_tag = false;
        let mut in_closing_tag = false;
        let mut expecting_tag_name = false;
        // Template expressions
        let mut expecting_expr_open = false;
        let mut in_expression = false;

        while let Some(c) = self.current_byte {
            if in_tag {
                match c {
                    b'>' | b'=' | b' ' => 'tag_name_end: {
                        if expecting_tag_name {
                            if !in_closing_tag {
                                // Check for a matching closing tag after
                                let tag_name = &self.s[tag_start + 1..self.i];
                                let mut closing_tag = String::with_capacity(3 + tag_name.len());
                                closing_tag += "</";
                                closing_tag += tag_name;
                                closing_tag += ">";
                                if !self.s[self.i..].contains(&closing_tag) {
                                    in_tag = false;
                                    break 'tag_name_end;
                                }
                            }
                            expecting_tag_name = false;
                        }

                        if c == b'>' {
                            self.i += 1;
                            self.current_byte = self.bytes.next();
                            while let Some(ws) = self.current_byte {
                                if char::from(ws).is_whitespace() {
                                    self.i += 1;
                                    self.current_byte = self.bytes.next();
                                    continue;
                                }
                                break;
                            }
                            return Some((&self.s[start..self.i], true));
                        } else if in_closing_tag {
                            // Invalid character
                            in_tag = false;
                        }
                    }
                    b'/' => {
                        if self.i == tag_start + 1 {
                            in_closing_tag = true;
                        } else if expecting_tag_name {
                            in_tag = false;
                        }
                    }
                    _ => {
                        if expecting_tag_name && !char::from(c).is_ascii_alphabetic() {
                            in_tag = false;
                        }
                    }
                }
            } else if in_expression {
                if c == b')' {
                    if !self.s[self.i..].contains(")") {
                        in_expression = false;
                    } else {
                        self.i += 1;
                        self.current_byte = self.bytes.next();
                        while let Some(ws) = self.current_byte {
                            if char::from(ws).is_whitespace() {
                                self.i += 1;
                                self.current_byte = self.bytes.next();
                                continue;
                            }
                            break;
                        }
                        return Some((&self.s[start..self.i], true));
                    }
                }
            } else if c == b'<' {
                if start == self.i {
                    in_tag = true;
                    expecting_tag_name = true;
                    tag_start = self.i;
                } else {
                    break;
                }
            } else if c == b'$' {
                expecting_expr_open = true;
            } else if c == b'(' {
                if expecting_expr_open {
                    if self.i != start + 1 {
                        self.i -= 1;
                        self.bytes = self.s.bytes();
                        self.current_byte = self.bytes.nth(self.i);
                        break;
                    }
                    in_expression = true;
                    expecting_expr_open = false;
                }
            } else if expecting_expr_open {
                expecting_expr_open = false;
            }

            self.i += 1;
            self.current_byte = self.bytes.next();
        }

        Some((&self.s[start..self.i], false))
    }
}

#[wasm_bindgen]
pub struct IsolateTagsSection {
    #[wasm_bindgen(getter_with_clone)]
    pub chunk: String,
    pub is_tag: bool,
}

#[wasm_bindgen(js_name = isolateTags)]
pub fn isolate_tags_owned(s: &str) -> Vec<IsolateTagsSection> {
    IsolateTags::new(s)
        .map(|(chunk, is_tag)| IsolateTagsSection {
            chunk: chunk.to_owned(),
            is_tag,
        })
        .collect()
}

fn custom_word_separator(line: &str) -> Box<dyn Iterator<Item = Word<'_>> + '_> {
    // Isolate tags and other text (e.g. ['test', '<size=16>', 'hello world', '</size>'])
    // Iter returns str slice and whether to separate words in the section
    // We're only breaking the string on ascii chars, so it's safe to use the bytes
    // iterator and split them based on the index.
    let mut isolate_iter = IsolateTags::new(line);

    let mut unicode_break_iter: Box<dyn Iterator<Item = Word<'_>> + '_> =
        Box::new(std::iter::empty());
    Box::new(std::iter::from_fn(move || {
        // Continue breaking current split
        let break_res = unicode_break_iter.next();
        if break_res.is_some() {
            return break_res;
        }

        // Advance to next (non-empty) split
        loop {
            if let Some((next_section, is_tag)) = isolate_iter.next() {
                if !is_tag {
                    let mut iter = UnicodeBreakProperties.find_words(next_section);
                    let break_res = iter.next();
                    if break_res.is_some() {
                        unicode_break_iter = iter;
                        return break_res;
                    }
                } else {
                    unicode_break_iter = Box::new(std::iter::empty());
                    return Some(Word::from(next_section));
                }
            } else {
                return None;
            }
        }
    }))
}

fn custom_wrap_algorithm<'a, 'b>(
    words: &'b [Word<'a>],
    line_widths: &'b [usize],
) -> Vec<&'b [Word<'a>]> {
    // Create intermediate buffer that doesn't contain formatting tags or expressions
    let mut clean_fragments = Vec::with_capacity(words.len());
    let mut removed_indices = Vec::with_capacity(words.len());
    let mut remove_offset = 0;
    for (i, word) in words.iter().enumerate() {
        let is_tag = word.starts_with("<") && word.ends_with(">");
        let is_expr = word.starts_with("$(") && word.ends_with(")");
        if is_tag || is_expr {
            removed_indices.push(i - remove_offset);
            remove_offset += 1;
            continue;
        }
        clean_fragments.push(words[i]);
    }

    // quick escape!!!11
    let f64_line_widths = line_widths.iter().map(|w| *w as f64).collect::<Vec<_>>();
    if remove_offset == 0 {
        return wrap_algorithms::wrap_optimal_fit(
            words,
            &f64_line_widths,
            &wrap_algorithms::Penalties::new(),
        )
        .unwrap();
    }

    // Wrap without formatting tags
    let wrapped = wrap_algorithms::wrap_optimal_fit(
        &clean_fragments,
        &f64_line_widths,
        &wrap_algorithms::Penalties::new(),
    )
    .unwrap();

    // Create results with formatting tags added back
    // Note: The break word option doesn't really affect the extra long lines since
    // the individual tags are separate words (it breaks words, not lines, duh)
    let mut lines = Vec::with_capacity(wrapped.len());
    let mut start = 0;
    let mut clean_start = 0;
    let mut removed_indices_i = 0;
    for (i, line) in wrapped.iter().enumerate() {
        let mut end: usize;
        if i == wrapped.len() - 1 {
            end = words.len();
        } else {
            let clean_end = clean_start + line.len();
            end = start + line.len();
            loop {
                let Some(index) = removed_indices.get(removed_indices_i) else {
                    break;
                };
                if *index >= clean_start {
                    if *index < clean_end {
                        end += 1;
                        removed_indices_i += 1;
                    } else {
                        break;
                    }
                }
            }
            clean_start = clean_end;
        }

        lines.push(&words[start..end]);
        start = end;
    }
    lines
}

pub fn wrap_text(
    string: &str,
    base_line_width: i32,
    line_width_multiplier: f32,
) -> Vec<Cow<'_, str>> {
    let line_width = (base_line_width as f32 * line_width_multiplier).round() as usize;
    let options = textwrap::Options::new(line_width)
        .word_separator(textwrap::WordSeparator::Custom(custom_word_separator))
        .wrap_algorithm(textwrap::WrapAlgorithm::Custom(custom_wrap_algorithm));
    textwrap::wrap(string, &options)
}

#[wasm_bindgen(js_name = wrapText)]
pub fn wrap_text_owned(
    string: &str,
    base_line_width: i32,
    line_width_multiplier: f32,
) -> Vec<String> {
    wrap_text(string, base_line_width, line_width_multiplier)
        .into_iter()
        .map(|s| s.into_owned())
        .collect()
}
