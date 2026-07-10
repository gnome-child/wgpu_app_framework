use unicode_segmentation::UnicodeSegmentation;

pub(super) fn normalize_single_line(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\r' | '\n' => ' ',
            _ => character,
        })
        .collect()
}

pub(super) fn normalize_multiline(text: &str) -> String {
    normalize_multiline_with_ending(text, "\n")
}

pub(super) fn normalize_multiline_with_ending(text: &str, ending: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' => {
                if characters.peek() == Some(&'\n') {
                    characters.next();
                }
                normalized.push_str(ending);
            }
            '\n' => {
                if characters.peek() == Some(&'\r') {
                    characters.next();
                }
                normalized.push_str(ending);
            }
            _ => normalized.push(character),
        }
    }
    normalized
}

pub(super) fn normalize_for_mode(multiline: bool, text: &str) -> String {
    if multiline {
        normalize_multiline(text)
    } else {
        normalize_single_line(text)
    }
}

pub(super) fn source_grapheme_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = vec![0];

    for (index, _) in text.grapheme_indices(true).skip(1) {
        boundaries.push(index);
    }

    if boundaries.last().copied() != Some(text.len()) {
        boundaries.push(text.len());
    }
    boundaries
}

pub(super) fn display_index(source_boundaries: &[usize], source_index: usize) -> usize {
    let source_index = floor_boundary_for_boundaries(source_boundaries, source_index);
    let character = source_boundaries
        .partition_point(|boundary| *boundary <= source_index)
        .saturating_sub(1);

    "•".len() * character
}

pub(super) fn floor_boundary_for_boundaries(boundaries: &[usize], index: usize) -> usize {
    boundaries
        .iter()
        .copied()
        .take_while(|boundary| *boundary <= index)
        .last()
        .unwrap_or(0)
}

pub(super) fn grapheme_range_in_text(
    text: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = range.start.min(range.end).min(text.len());
    let end = range.start.max(range.end).min(text.len());

    if start == end {
        let index = floor_grapheme_boundary(text, start);
        return index..index;
    }

    floor_grapheme_boundary(text, start)..ceil_grapheme_boundary(text, end)
}

pub(super) fn floor_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    floor_boundary_for_boundaries(&source_grapheme_boundaries(text), index)
}

pub(super) fn ceil_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    let boundaries = source_grapheme_boundaries(text);
    boundaries
        .iter()
        .copied()
        .find(|boundary| *boundary >= index)
        .unwrap_or(text.len())
}

pub(super) fn previous_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    source_grapheme_boundaries(text)
        .into_iter()
        .take_while(|boundary| *boundary < index)
        .last()
        .unwrap_or(0)
}

pub(super) fn next_grapheme_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    source_grapheme_boundaries(text)
        .into_iter()
        .find(|boundary| *boundary > index)
        .unwrap_or(text.len())
}

pub(super) fn word_boundaries(text: &str) -> Vec<usize> {
    let mut boundaries = vec![0];
    for (index, word) in text.split_word_bound_indices() {
        boundaries.push(index);
        boundaries.push(index + word.len());
    }
    for (index, _) in unicode_linebreak::linebreaks(text) {
        boundaries.push(index);
    }
    boundaries.push(text.len());
    boundaries.sort_unstable();
    boundaries.dedup();
    boundaries
}

pub(super) fn previous_word_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    word_boundaries(text)
        .into_iter()
        .take_while(|boundary| *boundary < index)
        .last()
        .unwrap_or(0)
}

pub(super) fn next_word_boundary(text: &str, index: usize) -> usize {
    let index = floor_boundary(text, index);
    word_boundaries(text)
        .into_iter()
        .find(|boundary| *boundary > index)
        .unwrap_or(text.len())
}

pub(super) fn word_range_at(text: &str, index: usize) -> std::ops::Range<usize> {
    let index = floor_boundary(text, index);
    for (start, word) in text.unicode_word_indices() {
        let end = start + word.len();
        if start <= index && index <= end {
            return start..end;
        }
    }
    let start = previous_word_boundary(text, index);
    let end = next_word_boundary(text, index);
    start..end
}

pub(super) fn floor_boundary(text: &str, index: usize) -> usize {
    let mut index = index.min(text.len());
    while index > 0 && !text.is_char_boundary(index) {
        index -= 1;
    }

    index
}
