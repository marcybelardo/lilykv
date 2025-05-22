use std::error::Error;

#[derive(Debug, PartialEq)]
enum RespType {
    SimpStr(String),
    Err(String),
    Int(i64),
    BulkStr(String),
    Arr(Vec<RespType>),
}

struct Parser;

impl Parser {
    pub fn deserialize(data: &str) -> Result<RespType, Box<dyn Error>> {
        Ok(RespType::SimpStr(String::from("hello world")))
    }
}

fn main() {
    let test = String::from("+hello world\r\n");
    if let Ok(msg) = Parser::deserialize(&test) {
        println!("{:?}", msg);
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
}
