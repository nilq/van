extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
fib : fun number -> number
fib = fun b: number -> number {
    return 10
}
    "#;

    let lexer      = make_lexer(source.clone().chars().collect());
    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Ok(ast) => println!("{:#?}", ast),
        Err(e)  => e.display(&source.lines().collect())
    }
}
