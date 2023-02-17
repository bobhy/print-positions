//! Demonstrating the convenience of grapheme cluster length arithmetic
//! when used for padding or filling fixed width fields
//! for display on a screen with monospace fonts and unicode + emoji support.

use anyhow::Result;
use print_positions::print_positions;


fn pad_field<'a>(input: &'a str, width: usize, fill: &str) {
    let padding = fill.repeat(width);
    let segs: Vec<_> = print_positions(input).collect();

    assert_eq!(
        input,
        segs.join(""),
        "print position segmentation doesn't lose / insert characters"
    );

    println!(
        "Content of this field is {} chars long but {} print positions wide",
        input.len(),
        segs.len(),
    );
    println!("   padded to width {width} with `{fill}`");
    println!("    {}{}", &padding[..(width - segs.len())], input);
    println!("    {}", padding);
}

fn main() -> Result<()> {
    println!("Padded field containing family emoji (3-code-point grapheme cluster, with zero width joiner)");
    // "family" emoji: woman ZWJ laptop == hacker? a single print position.
    pad_field("abc\u{1f469}\u{200d}\u{1f4bb}", 5, "+");
    println!(
        "Note:  emoji display is probably broken in your terminal window, few support ZWJ correctly."
    );
    println!("Try it in rust playground, or other web screen.\n");

    println!("Same padding works when coloring added.");
    let colorful = &vec![
        "a\u{1b}[30;42m",
        "b\u{1b}[30;45m",
        "c\u{1b}[0m",
        "\u{1f469}\u{200d}\u{1f4bb}",
    ]
    .join("");
    pad_field(colorful, 5, "+"); // extra rendering doesn't change padding

    Ok(())
}
