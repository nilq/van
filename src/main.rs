mod van;
use van::*;

fn main() {
    let source = r#"
function fib {
    | 0 -> 0
    | 1 -> match 1 {
        | 1 -> 2
    }
}"#;

    let lexer      = make_lexer(source.chars().collect());

    let lexed      = lexer.collect();
    
    println!("{:#?}", lexed);

    let traveler   = Traveler::new(lexed);
    let mut parser = Parser::new(traveler);
    
    println!("{:#?}", parser.parse());
}
