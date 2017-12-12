mod van;
use van::*;

fn main() {
    let source = r#"
fun fib a: i32 -> i32 {
    match a {
        | 0 -> 0
        | 1 -> 1
    }
}

function fib -> i32 {
    | 0 -> 0
    | 1 -> 1
}
"#;

    let lexer      = make_lexer(source.chars().collect());

    let lexed      = lexer.collect();
    
    println!("{:#?}", lexed);

    let traveler   = Traveler::new(lexed);
    let mut parser = Parser::new(traveler);
    
    println!("{:#?}", parser.parse());
}
