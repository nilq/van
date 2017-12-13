extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
struct Point {
    x: i32 y: i32
}
    "#;

    let lexer      = make_lexer(source.chars().collect());

    let lexed      = lexer.collect();

    println!("{:#?}", lexed);

    let traveler   = Traveler::new(lexed);
    let mut parser = Parser::new(traveler);
    
    println!("{:#?}", parser.parse());
}
