mod van;
use van::*;

fn main() {
    let source = r#"
match a {
    | 0 -> ^^a
    | 1 -> match 1 {
        | 0 -> a + 1
    }
}
    "#;

    let lexer      = make_lexer(source.chars().collect());

    let lexed      = lexer.collect();
    
    println!("{:#?}", lexed);

    let traveler   = Traveler::new(lexed);
    let mut parser = Parser::new(traveler);
    
    println!("{:#?}", parser.parse());
}
