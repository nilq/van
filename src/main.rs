mod van;
use van::*;

fn main() {
    let source = r#"
match a
  | 1 -> match a
    | 2 -> 3
    "#;

    let lexer      = make_lexer(source.chars().collect());

    let lexed      = lexer.collect();
    
    println!("{:#?}", lexed);

    let traveler   = Traveler::new(lexed);
    let mut parser = Parser::new(traveler);
    
    println!("{:#?}", parser.parse());
}
