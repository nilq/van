mod van;
use van::*;

fn main() {
    let source = r#"
mut a: i32 = -100
a = 10
    "#;
    
    let lexer      = make_lexer(source.chars().collect());
    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);
    
    println!("{:#?}", parser.parse());
}
