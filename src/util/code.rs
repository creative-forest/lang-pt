use super::{Code, Position};
use once_cell::unsync::OnceCell;

impl<'c> From<&'c [u8]> for Code<'c> {
    fn from(value: &'c [u8]) -> Self {
        Code::new(value)
    }
}
impl<'c> From<&'c str> for Code<'c> {
    fn from(value: &'c str) -> Self {
        Code::new(value.as_bytes())
    }
}

impl<'c> Code<'c> {
    pub fn new(value: &'c [u8]) -> Self {
        Self {
            value,
            line_breaks: OnceCell::new(),
        }
    }

    pub fn obtain_line_breaks(&self) -> &Vec<usize> {
        self.line_breaks.get_or_init(|| {
            self.value
                .iter()
                .enumerate()
                .filter_map(|(index, n)| if *n == b'\n' { Some(index) } else { None })
                .collect()
        })
    }
    pub fn obtain_position(&self, pointer: usize) -> Position {
        let line_breaks = self.obtain_line_breaks();
        let index = match line_breaks.binary_search(&pointer) {
            Ok(index) | Err(index) => index,
        };

        if index == 0 {
            let s = unsafe { std::str::from_utf8_unchecked(&self.value[..pointer]) };
            Position::new(1, s.len() + 1)
        } else {
            let break_point = line_breaks[index - 1] + 1;
            let s = unsafe { std::str::from_utf8_unchecked(&self.value[break_point..pointer]) };
            Position::new(index + 1, s.len() + 1)
        }
    }
}
