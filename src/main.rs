mod van;
use van::*;

fn main() {
    let source = r#"
fun fib n: i32 {
    match n {
        | 0 -> 0
        | 1 -> 1
        | n -> fib (n - 1) + fib (n - 2)
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
