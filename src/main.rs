extern crate colored;

mod van;
use van::*;

/* source.van

interface Debug {
    format: fun self -> string
}

struct Point {
    x: f64
    y: f64
}

implement Point as Debug {
    fun format self -> string {
        "(" ++ self.x ++ ", " ++ self.y ++ ")"
    }
}
*/

fn main() {
    let source = r#"
b: fun i32, mut uint -> i32
    "#;

    let lexer      = make_lexer(source.clone().chars().collect());
    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Ok(ast) => println!("{:#?}", ast),
        Err(e)  => e.display(&source.lines().collect())
    }
}
