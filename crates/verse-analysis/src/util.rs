pub fn get_word_at_cursor(line: &str, column: usize) -> Option<(&str, usize, usize)> {
    let bytes = line.as_bytes();
    if column >= bytes.len() {
        return None;
    }
    // Cursor must be on an identifier start character to extract a word.
    if !is_ident_start(bytes[column]) {
        return None;
    }
    let mut start = column;
    let mut end = column;
    while start > 0 && is_ident_char(bytes[start - 1]) {
        start -= 1;
    }
    while end < bytes.len() && is_ident_char(bytes[end]) {
        end += 1;
    }
    debug_assert!(start < end);
    Some((&line[start..end], start, end))
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_in_middle() {
        let (w, s, e) = get_word_at_cursor("foo Bar baz", 5).unwrap();
        assert_eq!(w, "Bar");
        assert_eq!(s, 4);
        assert_eq!(e, 7);
    }

    #[test]
    fn test_word_at_start() {
        let (w, _, _) = get_word_at_cursor("MyClass : something", 0).unwrap();
        assert_eq!(w, "MyClass");
    }

    #[test]
    fn test_cursor_on_space_returns_none() {
        assert!(get_word_at_cursor("foo bar", 3).is_none());
    }

    #[test]
    fn test_column_beyond_line_returns_none() {
        assert!(get_word_at_cursor("foo", 100).is_none());
    }
}
