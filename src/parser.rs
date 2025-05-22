use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum RespType {
    SimpStr(String),
    Err(String),
    Int(i64),
    BulkStr(String),
    Arr(Vec<RespType>),
}

pub struct Parser;

impl Parser {
    pub fn deserialize(data: &str) -> Result<RespType, Box<dyn Error>> {
        let (resp_type, contents) = data.split_at(1);

        match resp_type {
            "+" => Ok(RespType::SimpStr(contents.trim_end().to_owned())),
            "-" => Ok(RespType::Err(contents.trim_end().to_owned())),
            ":" => Ok(RespType::Int(contents.trim_end().parse::<i64>().unwrap())),
            "$" => Ok(RespType::BulkStr(contents.trim_end().to_owned())),
            "*" => {
                let (elems, arr_contents) = contents.split_once("\r\n").unwrap();
                let mut out: Vec<RespType> = Vec::new();
                let limit = elems.parse::<u32>().unwrap();
                let mut arr_lines = arr_contents.lines();

                for _ in 0..limit {
                    if let Some(line) = arr_lines.next() {
                        out.push(Parser::deserialize(line).unwrap());
                    }
                }

                Ok(RespType::Arr(out))
            }
            _ => Err("Type not implemented".into())
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
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), RespType::SimpStr(String::from("hello world")));
    }

    #[test]
    fn deserialize_error() {
        let test = String::from("-ERR test error\r\n");
        let parsed = Parser::deserialize(&test);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), RespType::Err(String::from("ERR test error")));
    }

    #[test]
    fn deserialize_int() {
        let test = String::from(":75\r\n");
        let parsed = Parser::deserialize(&test);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), RespType::Int(75));
    }

    #[test]
    fn deserialize_bulk_string() {
        let test = String::from("$0\r\n1\r\n");
        let parsed = Parser::deserialize(&test);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), RespType::BulkStr(String::from("0\r\n1")));
    }

    #[test]
    fn deserialize_array() {
        let test = String::from("*2\r\n+hello\r\n:7\r\n");
        let parsed = Parser::deserialize(&test);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), RespType::Arr(vec![RespType::SimpStr(String::from("hello")), RespType::Int(7)]));
    }
}
