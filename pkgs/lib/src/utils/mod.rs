#[cfg(test)]
pub mod test;

pub(crate) trait U8Ext {
    /// Returns whether this byte is a [C0 (U+0000..U+001F)](https://www.unicode.org/reports/tr44/#General_Category_Values)
    /// control character, which must be escaped in a string.
    /// \
    /// [RFC 8259: Escape in Json strings](https://www.rfc-editor.org/rfc/rfc8259#section-7)
    fn is_c0_control(self) -> bool;
}

impl U8Ext for u8 {
    fn is_c0_control(self) -> bool {
        self <= b'\x1f'
    }
}
