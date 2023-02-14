#[allow(unused_imports)]
use log::{error, info, trace, warn};

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

/// Iterator returns the print position string and its offset in the containing string.
///
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
pub fn new_printPositionIndices<'b>(s: &'b str) -> PrintPositionIndices<'b> {
    let iter = UnicodeSegmentation::grapheme_indices(s, true);
    PrintPositionIndices {
        string: s,
        cur_offset: 0,
        next_offset: 0,
        gi_iterator: iter,
    }
}

impl<'a> PrintPositionIndices<'a> {}

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
                "offset of retrieved grap not at start of rest of string"
            );
            self.next_offset += grap.1.len();

            let ascii_byte = grap.1.as_bytes()[0];

            match escape_state {
                EscapeState::Normal => {
                    if ascii_byte == 0x1b {
                        escape_state = EscapeState::EscapeSeen;
                    } else {
                        break;      // terminate the grapheme
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
                        warn!("unexpected char {ascii_byte} following ESC, terminating escape");
                        escape_state = EscapeState::Normal;
                    }
                },

                EscapeState::CSISeen => {
                    if (0x40..=0x7f).contains(&ascii_byte) {
                        // end of CSI, but continue accumulating
                        escape_state = EscapeState::Normal;
                    } else if (0x20..=0x3f).contains(&ascii_byte) { // accumulate CSI
                    } else {
                        warn!("unexpected char {ascii_byte} in CSI sequence, terminating escape");
                        escape_state = EscapeState::Normal;
                    }
                }
            }
        }

        // before returning, peek ahead and see whether there's a reset escape sequence we can append.
        // there are 3 ANSI reset sequences.
        // if, perversely, there is more than one in sequence, we'll just take one and
        // leave the others for the beginning of the next iteration.

        if self.next_offset < self.string.len() && self.string.as_bytes()[self.next_offset] == 0x1b
        {
            if self.next_offset + 2 <= self.string.len()
                && self.string[self.next_offset..].starts_with("\x1bc")
            {
                self.next_offset += 2;
                self.gi_iterator.next();
                self.gi_iterator.next();
            } else if self.next_offset + 3 <= self.string.len()
                && self.string[self.next_offset..].starts_with("\x1b[m")
            {
                self.next_offset += 3;
                self.gi_iterator.next();
                self.gi_iterator.next();
                self.gi_iterator.next();
            } else if self.next_offset + 4 <= self.string.len()
                && self.string[self.next_offset..].starts_with("\x1b[0m")
            {
                self.next_offset += 4;
                self.gi_iterator.next();
                self.gi_iterator.next();
                self.gi_iterator.next();
                self.gi_iterator.next();
            }
        }
        // return everything between start and end offsets
        let retlen = self.next_offset - self.cur_offset;
        if retlen <= 0 {
            return None;
        } else {
            let retval = (self.cur_offset, &self.string[self.cur_offset..self.next_offset]);
            // advance start to one beyond end of what we're returning
            self.cur_offset = self.next_offset;
            return Some(retval);
        }
    }
}

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

    fn run_test(input: &[&str], expected: &[(usize, &str)]) -> Result<()> {
        #[allow(unused_mut)]
        let mut test_input = input.join("");
        let mut observed: Vec<(usize, &str)> = vec![];

        let it = new_printPositionIndices(&test_input);

        for (offset, substring) in it {
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
        let e2 = ["f", esc_sgr_reset0()].join("");
        let expect = vec![(0, "a"), (1, "b"), (2, "c"), (3, &e1), (10, "e"), (11, "f")];

        run_test(&test_input, &expect)
    }
    #[test]
    fn trailing_reset() -> Result<()> {
        //let test_input = ["abc", esc_sgr_color(), "def", esc_sgr_reset0()];
        let test_input = [ "ef", esc_sgr_reset0()];
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
    
}
