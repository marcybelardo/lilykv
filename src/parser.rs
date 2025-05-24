#[derive(Debug, PartialEq)]
pub enum Resp {
    SimpStr(String),
    Err(String),
    Int(i64),
    BulkStr(String),
    Arr(Vec<Resp>),
}

pub struct ParserCursor<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> ParserCursor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input: input.as_bytes(), pos: 0 }
    }

    pub fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    pub fn step(&mut self, n: usize) {
        self.pos += n;
    }

    pub fn next_line(&mut self) -> Option<&'a str> {
        let slice = &self.input[self.pos..];
        let newline_pos = slice.windows(2).position(|w| w == b"\r\n")?;
        let line = &slice[..newline_pos];
        self.pos += newline_pos + 2;
        std::str::from_utf8(line).ok()
    }

    pub fn consume_bytes(&mut self, len: usize) -> Option<String> {
        let end = self.pos + len;
        if end + 2 > self.input.len() { return None }

        let data = &self.input[self.pos..end];
        let suffix = &self.input[end..end + 2];

        if suffix != b"\r\n" { return None }

        self.pos += end + 2;

        std::str::from_utf8(data).ok().map(|s| s.to_owned())
    }
}

pub struct Parser;

impl Parser {
    pub fn deserialize(data: String) -> Option<Resp> {
        let mut cursor = ParserCursor::new(&data);
        Self::parse_from_str(&mut cursor)
    }

    pub fn serialize(resp_type: Resp) -> Option<String> {
        match resp_type {
            Resp::SimpStr(s) => Some(format!("+{s}\r\n")),
            Resp::Err(s) => Some(format!("-{s}\r\n")),
            Resp::Int(n) => Some(format!(":{n}\r\n")),
            Resp::BulkStr(s) => Some(format!("${}\r\n{}\r\n", s.len(), s)),
            Resp::Arr(v) => {
                let mut out = String::new();
                let count = v.len();
                for elem in v {
                    out.push_str(&Self::serialize(elem)?);
                }
                Some(format!("*{}\r\n{}", count, out))
            }
        }
    }

    fn parse_from_str(cursor: &mut ParserCursor) -> Option<Resp> {
        match cursor.peek() {
            Some(b'+') => {
                cursor.step(1);
                Some(Resp::SimpStr(cursor.next_line()?.to_owned()))
            }
            Some(b'-') => {
                cursor.step(1);
                Some(Resp::Err(cursor.next_line()?.to_owned()))
            }
            Some(b':') => {
                cursor.step(1);
                cursor.next_line()?.parse::<i64>().ok().map(Resp::Int)
            }
            Some(b'$') => {
                cursor.step(1);
                let len = cursor.next_line()?.parse::<usize>().ok()?;
                let line = cursor.consume_bytes(len)?;
                Some(Resp::BulkStr(line))
            }
            Some(b'*') => {
                cursor.step(1);
                let elems = cursor.next_line()?.parse::<usize>().ok()?;
                let mut resp_vec: Vec<Resp> = Vec::with_capacity(elems);
                for _ in 0..elems {
                    let elem = Self::parse_from_str(cursor)?;
                    println!("Pushing {:?}", elem);
                    resp_vec.push(elem);
                }

                Some(Resp::Arr(resp_vec))
            }
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_simple_string() {
        let test = String::from("+OK\r\n");
        let expected = Some(Resp::SimpStr(String::from("OK")));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_error() {
        let test = String::from("-ERR something bad\r\n");
        let expected = Some(Resp::Err(String::from("ERR something bad")));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_integer() {
        let test = String::from(":42\r\n");
        let expected = Some(Resp::Int(42));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_bulk_string() {
        let test = String::from("$6\r\nfoobar\r\n");
        let expected = Some(Resp::BulkStr(String::from("foobar")));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_array_of_simple_strings() {
        let test = String::from("*2\r\n+foo\r\n+bar\r\n");
        let expected = Some(Resp::Arr(vec![
            Resp::SimpStr(String::from("foo")),
            Resp::SimpStr(String::from("bar")),
        ]));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_array_of_mixed_types() {
        let test = String::from("*3\r\n+hello\r\n:123\r\n$5\r\nworld\r\n");
        let expected = Some(Resp::Arr(vec![
            Resp::SimpStr(String::from("hello")),
            Resp::Int(123),
            Resp::BulkStr(String::from("world")),
        ]));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_nested_array() {
        let test = String::from("*2\r\n*2\r\n+foo\r\n+bar\r\n:42\r\n");
        let expected = Some(Resp::Arr(vec![
            Resp::Arr(vec![
                Resp::SimpStr(String::from("foo")),
                Resp::SimpStr(String::from("bar")),
            ]),
            Resp::Int(42),
        ]));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_empty_array() {
        let test = String::from("*0\r\n");
        let expected = Some(Resp::Arr(vec![]));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_empty_bulk_string() {
        let test = String::from("$0\r\n\r\n");
        let expected = Some(Resp::BulkStr(String::new()));
        assert_eq!(Parser::deserialize(test), expected);
    }

    #[test]
    fn deserialize_invalid_prefix() {
        let test = String::from("!oops\r\n");
        assert_eq!(Parser::deserialize(test), None);
    }

    #[test]
    fn deserialize_invalid_integer() {
        let test = String::from(":notanint\r\n");
        assert_eq!(Parser::deserialize(test), None);
    }

    #[test]
    fn deserialize_truncated_bulk_string() {
        let test = String::from("$4\r\nfoo\r\n"); // Length is 4 but only 3 bytes in "foo"
        assert_eq!(Parser::deserialize(test), None);
    }

    #[test]
    fn serialize_simple_string() {
        let data = Resp::SimpStr(String::from("OK"));
        let test = Parser::serialize(data);
        assert_eq!(test, Some(String::from("+OK\r\n")));
    }

    #[test]
    fn serialize_error() {
        let data = Resp::Err(String::from("ERR unknown command"));
        let test = Parser::serialize(data);
        assert_eq!(test, Some(String::from("-ERR unknown command\r\n")));
    }

    #[test]
    fn serialize_integer() {
        let data = Resp::Int(1000);
        let test = Parser::serialize(data);
        assert_eq!(test, Some(String::from(":1000\r\n")));
    }

    #[test]
    fn serialize_bulk_string() {
        let data = Resp::BulkStr(String::from("foobar"));
        let test = Parser::serialize(data);
        assert_eq!(test, Some(String::from("$6\r\nfoobar\r\n")));
    }

    #[test]
    fn serialize_empty_bulk_string() {
        let data = Resp::BulkStr(String::from(""));
        let test = Parser::serialize(data);
        assert_eq!(test, Some(String::from("$0\r\n\r\n")));
    }

    #[test]
    fn serialize_array_of_integers() {
        let data = Resp::Arr(vec![
            Resp::Int(1),
            Resp::Int(2),
            Resp::Int(3),
        ]);
        let test = Parser::serialize(data);
        assert_eq!(test, Some(String::from("*3\r\n:1\r\n:2\r\n:3\r\n")));
    }

    #[test]
    fn serialize_array_of_mixed_types() {
        let data = Resp::Arr(vec![
            Resp::SimpStr(String::from("hello")),
            Resp::Int(42),
            Resp::BulkStr(String::from("foo")),
        ]);
        let test = Parser::serialize(data);
        assert_eq!(test, Some(String::from("*3\r\n+hello\r\n:42\r\n$3\r\nfoo\r\n")));
    }

    #[test]
    fn serialize_nested_array() {
        let data = Resp::Arr(vec![
            Resp::Int(1),
            Resp::Arr(vec![
                Resp::BulkStr(String::from("foo")),
                Resp::BulkStr(String::from("bar")),
            ]),
            Resp::Int(2),
        ]);
        let test = Parser::serialize(data);
        assert_eq!(
            test,
            Some(String::from("*3\r\n:1\r\n*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n:2\r\n"))
        );
    }
}
