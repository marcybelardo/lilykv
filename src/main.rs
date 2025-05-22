mod parser;
use crate::parser::Parser;

fn main() {
    let test = String::from("+hello world\r\n");
    if let Ok(msg) = Parser::deserialize(&test) {
        println!("{:?}", msg);
    }
}
