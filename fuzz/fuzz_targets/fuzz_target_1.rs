#![no_main]
extern crate print_positions;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {

    let mut last_offset = 0;
    let mut out_grap = "".to_string();

    if let Ok(s) = std::str::from_utf8(data) {

        for (offset, grap) in print_positions::print_position_indices(s) {
            assert!(offset < s.len());
            assert!((last_offset == 0  && offset == 0 ) || offset > last_offset, "current offset {offset} not > previous {last_offset}");
            
            last_offset = offset;
            out_grap.push_str(grap);
        }

        assert_eq!(s, out_grap, "catenated output not == input")
    }
});
