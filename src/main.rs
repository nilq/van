extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
interface Debug {
    debug: fun -> string
    grr:   fun uint -> uint
}

struct Point {
    x: float
    y: float
}

implement Point as Debug {
    fun debug -> string {
        "no bugs"
    }
    
    fun grr a: uint -> uint {
        b := a + 10
        b
    }
}

a: [uint; 3 + 2] = [1, 2, 3,]
    "#;

    let lexer      = make_lexer(source.clone().chars().collect());
    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Ok(ast) => println!("{:#?}", ast),
        Err(e)  => e.display(&source.lines().collect())
    }
}
