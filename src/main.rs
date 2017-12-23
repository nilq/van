extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
keypressed: mut fun string bool = extern love.draw

extern import deepcopy expose (deepcopy)

extern struct love {
    load:   mut fun
    draw:   mut fun
    update: mut fun mut number
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
