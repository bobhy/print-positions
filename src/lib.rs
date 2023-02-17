use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

/// Iterator which retuns "print positions" found in a string.
/// ```rust
/// use print_positions::print_positions;
///
/// // Segmentation via extended grapheme clusters
/// let segs: Vec<_> = print_positions("abc\u{1f468}\u{200d}\u{1f467}\u{200d}\u{1f466}").collect();
/// assert_eq!(vec!("a","b","c", "\u{1f468}\u{200d}\u{1f467}\u{200d}\u{1f466}"), segs);
///
/// // Control chars and ANSI escapes *included within* grapheme clusters
/// let segs: Vec<_> = print_positions("abc\u{1b}[37;46mdef\u{1b}[0mg").collect();
/// assert_eq!(vec!("a","b","c", "\u{1b}[37;46md","e","f\u{1b}[0m", "g"), segs);
/// ```
///
/// Also see [crate::examples/padding] for performing
/// fixed-width formatting based on print positions in the data
/// rather than its string length.
///
pub struct PrintPositions<'a>(PrintPositionIndices<'a>);

impl<'a> PrintPositions<'a> {
    /// View the underlying data (the part yet to be iterated) as a slice of the original string.
    ///
    /// ```rust
    /// # use print_positions::print_positions;
    /// let mut iter = print_positions("abc");
    /// assert_eq!(iter.as_str(), "abc");
    /// iter.next();
    /// assert_eq!(iter.as_str(), "bc");
    /// iter.next();
    /// iter.next();
    /// assert_eq!(iter.as_str(), "");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &'a str {
        &self.0.string[self.0.cur_offset..self.0.string.len()]
    }
}

impl<'a> Iterator for PrintPositions<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((_, s)) = self.0.next() {
            Some(s)
        } else {
            None
        }
    }
}

#[inline]
pub fn print_positions<'a>(s: &'a str) -> PrintPositions<'a> {
    PrintPositions(print_position_indices(s))
}
/// Iterator returns "print positions"
/// and their offsets in the containing string.
/// ```rust
/// use print_positions::print_position_indices;
///
/// // Segmentation via extended grapheme clusters
/// let segs: Vec<(usize, &str)> = print_position_indices("\u{1f468}\u{200d}\u{1f467}\u{200d}\u{1f466}abc").collect();
/// assert_eq!(vec!((0, "\u{1f468}\u{200d}\u{1f467}\u{200d}\u{1f466}"), (18, "a"),(19, "b"),(20, "c"),), segs);
///
/// // Control chars and ANSI escapes *included within* grapheme clusters
/// let segs: Vec<_> = print_position_indices("\u{1b}[37;46mdef\u{1b}[0mg").collect();
/// assert_eq!(vec!((0, "\u{1b}[37;46md"),(9, "e"), (10, "f\u{1b}[0m"), (15, "g")), segs);
/// ```

#[derive(Clone)]
pub struct PrintPositionIndices<'a> {
    // the victim string -- all outputs are slices of this.
    string: &'a str,
    // offset of beginning of slice currently being assembled or last returned.
    cur_offset: usize,
    // offset of the first unexamined char
    next_offset: usize,
    // wrapped grapheme (== extended grapheme cluster) iterator
    gi_iterator: GraphemeIndices<'a>,
}
/// Factory method to create a new [PrintPositionIndices] iterator
///
#[inline]
pub fn print_position_indices<'a>(s: &'a str) -> PrintPositionIndices<'a> {
    let iter = UnicodeSegmentation::grapheme_indices(s, true);
    PrintPositionIndices {
        string: s,
        cur_offset: 0,
        next_offset: 0,
        gi_iterator: iter,
    }
}

impl<'a> PrintPositionIndices<'a> {
    /// View the underlying data (the part yet to be iterated) as a slice of the original string.
    ///
    /// ```rust
    /// # use print_positions::print_position_indices;
    /// let mut iter = print_position_indices("abc");
    /// assert_eq!(iter.as_str(), "abc");
    /// iter.next();
    /// assert_eq!(iter.as_str(), "bc");
    /// iter.next();
    /// iter.next();
    /// assert_eq!(iter.as_str(), "");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &'a str {
        &self.string[self.cur_offset..self.string.len()]
    }
}

impl<'a> Iterator for PrintPositionIndices<'a> {
    type Item = (usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_offset > self.string.len() {
            return None;
        };

        enum EscapeState {
            Normal,
            EscapeSeen, // just saw an escape, start accumulating
            CSISeen,    // 2nd char not terminal, continue accumulating
        }

        let mut escape_state = EscapeState::Normal;

        while self.next_offset < self.string.len() {
            let grap = self.gi_iterator.next().expect("already checked not at EOS");
            debug_assert_eq!(
                grap.0, self.next_offset,
                "offset of retrieved grap {} not at start of rest of string {}", grap.0, self.next_offset
            );
            self.next_offset += grap.1.len();

            let ascii_byte = grap.1.as_bytes()[0];

            match escape_state {
                EscapeState::Normal => {
                    if ascii_byte == 0x1b {
                        escape_state = EscapeState::EscapeSeen;
                    } else {
                        break; // terminate the grapheme
                    }
                }

                EscapeState::EscapeSeen => match ascii_byte {
                    b'[' => {
                        escape_state = EscapeState::CSISeen;
                    }
                    0x40..=0x5F => {
                        // terminate escape, but continue accumulating
                        escape_state = EscapeState::Normal;
                    }
                    _ => {
                        debug_assert!(
                            true, // don't actually fail fuzz testing, but document behavior for malformed escapes.
                            "unexpected char {ascii_byte} following ESC, terminating escape"
                        );
                        escape_state = EscapeState::Normal;
                    }
                },

                EscapeState::CSISeen => {
                    if (0x40..=0x7f).contains(&ascii_byte) {
                        // end of CSI, but continue accumulating
                        escape_state = EscapeState::Normal;
                    } else if (0x20..=0x3f).contains(&ascii_byte) { // accumulate CSI
                    } else {
                        debug_assert!(
                            true, // don't actually fail fuzz testing, but document behavior for malformed escapes.
                            "unexpected char {ascii_byte} in CSI sequence, terminating escape"
                        );
                        escape_state = EscapeState::Normal;
                    }
                }
            }
        }

        // before returning, peek ahead and see whether there's a reset escape sequence we can append.
        // there are 3 ANSI reset sequences.
        // if, perversely, there is more than one in sequence, we'll just take one and
        // leave the others for the beginning of the next iteration.
        // If, even more perversely, the last char of the esc sequence plus some following
        // characters in the string form a multi-character grapheme, take all of that.
        // Hooray for fuzz testing.

        while self.next_offset < self.string.len()
            && self.string.as_bytes()[self.next_offset] == 0x1b
        {
            if self.next_offset + 2 <= self.string.len()
                && self.string[self.next_offset..].starts_with("\x1bc")
            {
                self.gi_iterator.next();
                let last = self.gi_iterator.next().expect("must be >=2");
                self.next_offset += 1 + last.1.len();
            } else if self.next_offset + 3 <= self.string.len()
                && self.string[self.next_offset..].starts_with("\x1b[m")
            {
                self.gi_iterator.next();
                self.gi_iterator.next();
                let last = self.gi_iterator.next().expect("must be >=3");
                self.next_offset += 2 + last.1.len();
            } else if self.next_offset + 4 <= self.string.len()
                && self.string[self.next_offset..].starts_with("\x1b[0m")
            {
                self.gi_iterator.next();
                self.gi_iterator.next();
                self.gi_iterator.next();
                let last = self.gi_iterator.next().expect("must be >=4");
                self.next_offset += 3 + last.1.len();
            } else {
                break; // not one we're interested in at end of grap.
            }
        }
        // return everything between start and end offsets
        let retlen = self.next_offset - self.cur_offset;
        if retlen <= 0 {
            return None;
        } else {
            let retval = (
                self.cur_offset,
                &self.string[self.cur_offset..self.next_offset],
            );
            // advance start to one beyond end of what we're returning
            self.cur_offset = self.next_offset;
            return Some(retval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use anyhow::{anyhow, Result};

    fn esc_sgr_reset0() -> &'static str {
        "\x1b[0m"
    }
    fn esc_sgr_reset() -> &'static str {
        "\x1b[m"
    }
    fn esc_sgr_color() -> &'static str {
        "\x1b[1;3m"
    }

    // test both flavors of iterator for one scenario
    fn run_test(input: &[&str], expected: &[(usize, &str)]) -> Result<()> {
        #[allow(unused_mut)]
        let mut test_input = input.join("");
        let mut observed: Vec<(usize, &str)> = vec![];

        for (offset, substring) in print_position_indices(&test_input) {
            if observed.len() > 0 {
                let last_off = observed.last().expect("length checked").0;
                assert!(
                    offset > last_off,
                    "new offset({offset}) not greater than last seen ({last_off})"
                );
            };
            assert!(substring.len() > 0, "empty substring returned");
            observed.push((offset, substring));
        }

        assert_eq!(expected, observed);

        let mut observed: Vec<&str> = vec![];

        for substring in print_positions(&test_input) {
            assert!(
                substring.len() > 0,
                "empty substring returned (print_positions)"
            );
            observed.push(substring);
        }

        assert_eq!(
            expected.len(),
            observed.len(),
            "comparing print positions iterator length"
        );
        for (exp, obs) in expected.iter().zip(observed) {
            assert_eq!(exp.1, obs, "comparing print positions individual returns");
        }

        Ok(())
    }

    #[test]
    fn empty_string() -> Result<()> {
        run_test(&vec![], &vec![])
    }
    #[test]
    fn simple1() -> Result<()> {
        //let test_string = ["abc", esc_sgr_color(), "def", esc_sgr_reset0()].join("");
        let test_input = ["abc", esc_sgr_color(), "def"];
        let e1 = [esc_sgr_color(), "d"].join("");
        let expect = vec![(0, "a"), (1, "b"), (2, "c"), (3, &e1), (10, "e"), (11, "f")];

        run_test(&test_input, &expect)
    }
    #[test]
    fn trailing_reset() -> Result<()> {
        //let test_input = ["abc", esc_sgr_color(), "def", esc_sgr_reset0()];
        let test_input = ["ef", esc_sgr_reset0()];
        let e2 = ["f", esc_sgr_reset0()].join("");
        //let expect = vec![(0, "a"), (1, "b"), (2, "c"), (3, &e1), (10, "e"), (11, "f"), (12, &e2)];
        let expect = vec![(0, "e"), (1, &e2)];

        run_test(&test_input, &expect)
    }
    #[test]
    fn embedded_csi_and_trailing_reset() -> Result<()> {
        let test_input = ["abc", esc_sgr_color(), "def", esc_sgr_reset()];
        //let test_input = [ "f", esc_sgr_reset0()];
        let e1 = [esc_sgr_color(), "d"].join("");
        let e2 = ["f", esc_sgr_reset()].join("");
        let expect = vec![(0, "a"), (1, "b"), (2, "c"), (3, &e1), (10, "e"), (11, &e2)];
        //let expect = vec![(0, &e2)];

        run_test(&test_input, &expect)
    }

    #[test]
    fn double_trailing_reset() -> Result<()> {
        let test_input = [
            "abc",
            esc_sgr_color(),
            "def",
            esc_sgr_reset(),
            esc_sgr_reset0(),
            "g",
        ];
        let e1 = [esc_sgr_color(), "d"].join("");
        let e2 = ["f", esc_sgr_reset(), esc_sgr_reset0()].join("");
        let expect = vec![
            (0, "a"),
            (1, "b"),
            (2, "c"),
            (3, &e1),
            (10, "e"),
            (11, &e2),
            (19, "g"),
        ];

        run_test(&test_input, &expect)
    }

    // fuzz testing found a problem with this input: [45, 27, 91, 109, 221, 133]
    // but it doesn't fail in test even with nightly compiler --release vs --test?
    #[test]
    fn error_from_fuzz_test() -> Result<()> {
        //let test_input = ["-\x1b[0m\u{dd}\u{85}"];
        let input = ["\u{d1}\u{97}\x1b[m\u{d2}\u{83}"];
        let expected = vec![
            (0, "\u{d1}"),
            (2, "\u{97}\x1b[m"),
            (7, "\u{d2}"),
            (9, "\u{83}"),
        ];

        run_test(&input, &expected)
    }

    #[test]
    fn new_line_tests() -> Result<()> {
        // unicode standard says \r\n is a single grapheme.  But separately? or \n\r?
        let input = ["\r\n", "\na\rb", "\n\r", "\r\n"];
        let expected = vec![
            (0, "\r\n"),
            (2, "\n"),
            (3, "a"),
            (4, "\r"),
            (5, "b"),
            (6, "\n"),
            (7, "\r"),
            (8, "\r\n"),
        ];

        run_test(&input, &expected)
    }

    // testing the whole zoo of Unicode is somebody else's problem
    // but we do test at least test some multi-byte unicode and some grapheme clusters

    #[test]
    fn unicode_multibyte_mixed_tests() -> Result<()> {
        // samples from UnicodeSegmentation "a̐éö̲"; // 3 bytes each

        let input = ["a", esc_sgr_color(), "a̐é", esc_sgr_reset(), esc_sgr_reset()];
        let e1 = [esc_sgr_color(), "a̐"].join("");
        let e2 = ["é", esc_sgr_reset(), esc_sgr_reset()].join("");
        let expected = vec![(0, "a"), (1, &e1), (10, &e2)];

        run_test(&input, &expected)
    }

    #[test]
    fn fuzz_failure_1() -> Result<()> {
        // hooray for fuzz testing!
        // it turns out the last char of a reset escape sequence
        // can form a grapheme cluster with the following chars.
        // so the reset sequence may *not* be the end of the returned grapheme.
        let input = [std::str::from_utf8(&[63, 27, 99, 217, 151]).expect("foo"),];
        let expected = vec![(0, "?\u{1b}c\u{657}")];

        run_test(&input, &expected)

    }
}
