extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
unless 1 + 10 == 12 {
    "the world is ok" |> print
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
