# Crate print_positions
A set of iterators which return the characters making up a "print positions", rather 
than the individual characters.

A print position is a potentially multi-byte slice of string which a user would read as a single 'character' on
the screen or printed page. This print position contains a *single* 
[UAX#29 extended grapheme cluster](http://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries), possibly surrounded by 
[ANSI escape codes](https://en.wikipedia.org/wiki/ANSI_escape_code#Description).  Escape sequences and control characters
are assumed to not consume visible space.

This is handy for laying out text on an Ansi-compatible console screen,
where ANSI SGR codes are used to set text colors and intensity and the text
is non-ASCII unicode.  You can count the returned graphemes as one column each
and pad the rest of the field with blanks to the desired visual width.
 
Most ANSI sequences change the rendering of the characters that *follow* them, so they are
embedded *at the beginning* of the returned grapheme slice.  
However, codes that *reset* special rendering modes, such as SGR reset and RIS,
are embedded *at the end*.  
This allows caller to insert an undecorated string into the middle of a
decorated string (e.g, SGR _something_ _text_ SGR RESET)  without having to track the rendering state, so long as the
insertion is done on print position boundaries.

But some ANSI sequences and control codes in fact *do* move the cursor or otherwise consume a print position
and will be iterated out as part of some returned print position.
This package does not attempt to track the screen cursor position,  we blithely assume such 
sequences will not be input to the iterator.
This is arguably a bug 
in the case of backspace, tab, newline, CUP and several more.  PRs or simple suggestions for improvement are welcome!
