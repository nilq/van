extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
struct Point {
    x: int
    y: int
}

pos: Point = new {
    x = 10
    y = 10
}

pos2 := new Point {
    x = 100
    y = 100
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
