#[derive(Debug, PartialEq)]
pub enum Resp {
    SimpStr(String),
    Err(String),
    Int(i64),
    BulkStr(String),
    Arr(Vec<Resp>),
}

pub struct Parser;

impl Parser {
    pub fn deserialize(data: &str) -> Option<Resp> {
        let mut input = data;
        Self::parse(&mut input)
    }

    fn parse(input: &mut &str) -> Option<Resp> {
        let (prefix, rest) = input.split_at(1);
        *input = rest;

        match prefix {
            "+" => Some(Resp::SimpStr(Self::read_line(input)?)),
            "-" => Some(Resp::Err(Self::read_line(input)?)),
            ":" => Self::read_line(input)?.parse::<i64>().ok().map(Resp::Int),
            "$" => {
                let len_line = Self::read_line(input)?;
                let len: usize = len_line.parse().ok()?;
                let (bulk, rest) = input.split_at(len + 2);
                let (bulk_str, term) = bulk.split_at(len);
                if term != "\r\n" {
                    return None;
                }
                *input = rest;
                Some(Resp::BulkStr(bulk_str.to_owned()))
            }
            "*" => {
                let count_line = Self::read_line(input)?;
                let count: usize = count_line.parse().ok()?;
                let mut elems = Vec::with_capacity(count);
                for _ in 0..count {
                    let elem = Self::parse(input)?;
                    elems.push(elem);
                }
                Some(Resp::Arr(elems))
            }
            _ => None,
        }
    }

    fn read_line(input: &mut &str) -> Option<String> {
        if let Some(pos) = input.find("\r\n") {
            let (line, rest) = input.split_at(pos);
            *input = &rest[2..];
            Some(line.to_owned())
        } else {
            None
        }
    }

    pub fn serialize(data: Resp) -> Option<String> {
        match data {
            Resp::SimpStr(str) => Some(format!("+{str}\r\n")),
            Resp::Err(str) => Some(format!("-{str}\r\n")),
            Resp::Int(num) => Some(format!(":{num}\r\n")),
            Resp::BulkStr(str) => Some(format!("${}\r\n{}\r\n", str.len(), str)),
            Resp::Arr(elems) => {
                let mut out = String::new();
                let mut count = 0;
                for elem in elems {
                    if let Some(serialized) = Parser::serialize(elem) {
                        out.push_str(&format!("{}\r\n", serialized));
                    } else {
                        return None;
                    }
                    count += 1;
                }
                out.insert_str(0, &format!("*{count}\r\n"));

                Some(out)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_simple_string() {
        let test = String::from("+hello world\r\n");
        let parsed = Parser::deserialize(&test);
        assert_eq!(parsed, Some(Resp::SimpStr(String::from("hello world"))));
    }

    #[test]
    fn deserialize_error() {
        let test = String::from("-ERR test error\r\n");
        let parsed = Parser::deserialize(&test);
        assert_eq!(parsed, Some(Resp::Err(String::from("ERR test error"))));
    }

    #[test]
    fn deserialize_int() {
        let test = String::from(":75\r\n");
        let parsed = Parser::deserialize(&test);
        assert_eq!(parsed, Some(Resp::Int(75)));
    }

    #[test]
    fn deserialize_bulk_string() {
        let test = String::from("$12\r\n1\r\nanother\r\n");
        let parsed = Parser::deserialize(&test);
        assert_eq!(parsed, Some(Resp::BulkStr(String::from("1\r\nanother\r\n"))));
    }

    #[test]
    fn deserialize_array() {
        let test = String::from("*2\r\n+hello\r\n:7\r\n");
        let parsed = Parser::deserialize(&test);
        assert_eq!(parsed, Some(Resp::Arr(vec![Resp::SimpStr(String::from("hello")), Resp::Int(7)])));
    }

    #[test]
    fn serialize_simple_string() {
        let test = Resp::SimpStr(String::from("hello!"));
        let parsed = Parser::serialize(test);
        assert_eq!(parsed, Some(String::from("+hello!\r\n")));
    }

    #[test]
    fn serialize_error() {
        let test = Resp::Err(String::from("Bad error"));
        let parsed = Parser::serialize(test);
        assert_eq!(parsed, Some(String::from("-Bad error\r\n")));
    }
}
