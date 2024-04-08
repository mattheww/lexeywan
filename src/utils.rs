//! Common utility code.

//! Returns an escaped form of a string, for display to an end user.
//!
//! Uses nonascii characters to indicate the escapes, to avoid conflicts with characters meaningful
//! in Rust.
pub fn escape_for_display(input: &str) -> String {
    let mut s = String::new();
    for c in input.chars() {
        if c.is_ascii_graphic() || c == ' ' {
            s.push(c)
        } else if (c as u32) < 256 {
            s.push_str(&format!("‹{:02X}›", c as u32));
        } else {
            s.push_str(&format!("‹{:04X}›", c as u32));
        }
    }
    s
}
