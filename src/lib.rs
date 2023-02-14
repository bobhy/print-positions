use simple_logging;
use log::LevelFilter;

pub use unicode_segmentation::UnicodeSegmentation;

fn main() {

    simple_logging::log_to_stderr(LevelFilter::Info);

}

pub mod print_position;

/// A print position is a slice of string which a user would read as a single 'character'
/// occupying one visible position on a printed page or screen.  
/// The slice contains an extended grapheme cluster as defined by 
/// [UAX#29](http://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries)
/// surrounded by any 
/// [ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#Description)
/// found in the string that do not consume visible space.  
/// 
/// This is a handy concept for laying out text in an Ansi-compatible console screen, 
/// where ANSI SGR codes are used to set text colors and intensity.  
/// The length on screen of a print position is 1, by definition.
/// You can use arithmetic based on these lengths to pad and align text on the page or screen.
///  
/// With the presumption that most codes change the rendering of the characters that *follow*,
/// most codes are embeded *at the beginning* of the [PrintPosition].  
/// However, codes that *reset* special rendering modes, such as SGR reset and RIS,
/// are embedded *at the end* of the [PrintPosition].  
/// This allows caller to insert an undecorated string into the middle of a 
/// decorated string without having to track the rendering state, so long as the 
/// insertion is done on print position boundaries.
/// 
/// However, ANSI control codes and escape sequences can move the cursor on a compatible device.
/// These characters will be passed through without modification to the output slice,
/// but their rendering effect will not be accounted for in the print position arithmetic.
/// This handling for backspace and tab is arguably a bug.  
/// 
/// Implementation note:
/// Should there be an iterator which explicitly yields [PrintPosition]? 
/// It is defined here primarily as a pedagogic device, to explain the iterator logic.
/// But the iterator itself returns slice of string for efficiency.
/// 
/// 

/*
pub struct PrintPositionIndices<'a>(&'a str);

/// Iterate over input string, yielding [&str] which segment it on [PrintPosition] boundaries.
pub trait PrintPositionSegmentation {
    /// Returns an iterator over the print positions found in a string.
    fn PrintPositionIndices<'a>(&'a self) -> PrintPositionIndices<'a> ;
}

impl PrintPositionSegmentation for str {
    fn PrintPositionIndices<'a>(&'a self) -> PrintPositionIndices<'a> {
         new_printPositionIndices(self)
    }

    
}
*/
#[cfg(test)]
mod tests {
    use super::*;

    
}
