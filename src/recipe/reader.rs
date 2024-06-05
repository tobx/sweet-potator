use std::{
    io::{self, BufRead},
    ops::Not,
};

pub(super) struct Reader<R> {
    lines: io::Lines<io::BufReader<R>>,
    should_trim: bool,
}

impl<R: io::Read> Reader<R> {
    pub fn new(reader: R, should_trim: bool) -> Self {
        let lines = io::BufReader::new(reader).lines();
        Self { lines, should_trim }
    }

    pub fn next_block(&mut self) -> io::Result<Option<Vec<String>>> {
        let trim = |s: &str| (if self.should_trim { s.trim() } else { s }).to_string();
        let block: Vec<_> = self
            .lines
            .by_ref()
            .map(|line| Ok(trim(&line?)))
            .skip_while(|res| matches!(res, Ok(line) if line.is_empty()))
            .take_while(|res| matches!(res, Ok(line) if !line.is_empty()))
            .collect::<io::Result<_>>()?;
        Ok(block.is_empty().not().then_some(block))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_normal() {
        let text = [
            "",
            "",
            "block 1, line 1",
            "block 1, line 2",
            " ",
            "",
            "",
            " ",
            "",
            "block 2, line 1",
            "",
            "",
        ]
        .join("\n");
        let mut reader = Reader::new(io::Cursor::new(text), false);
        assert_eq!(
            reader.next_block().unwrap().unwrap(),
            vec!["block 1, line 1", "block 1, line 2", " "]
        );
        assert_eq!(reader.next_block().unwrap().unwrap(), vec![" "]);
        assert_eq!(
            reader.next_block().unwrap().unwrap(),
            vec!["block 2, line 1"]
        );
        assert_eq!(reader.next_block().unwrap(), None);
    }

    #[test]
    fn test_trimmed() {
        let text = [" ", "block", " ", "", " ", ""].join("\n");
        let mut reader = Reader::new(io::Cursor::new(text), true);
        assert_eq!(reader.next_block().unwrap().unwrap(), vec!["block"]);
        assert_eq!(reader.next_block().unwrap(), None);
    }
}
