#![no_main]
extern crate print_positions;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {

    let mut prev_offset = 0;
    let mut out_grap = "".to_string();

    if let Ok(s) = std::str::from_utf8(data) {

        for (start, end) in print_positions::print_positions(s) {
            assert!(end <= s.len());
            assert!(end > start);
            assert!((prev_offset == 0  && start == 0 ) || start >= prev_offset, "current offset {start} not > previous {prev_offset}");
            
            prev_offset = end;
            out_grap.push_str(&s[start .. end]);
        }

        assert_eq!(s, out_grap, "catenated output not == input")
    }
});
