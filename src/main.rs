extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
struct number {}
struct nil    {}
struct string {}

struct Point {
    x: number
    y: number
}

mut outer := 10

fun foo x: number y: number -> Point {
    mut a: Point = new Point {
        x = x
        y = y
    }

    a.x = 100

    a
}

b: Point = foo 100 100
    "#;

    let lexer      = make_lexer(source.clone().chars().collect());
    let traveler   = Traveler::new(lexer.collect());
    let mut parser = Parser::new(traveler);

    match parser.parse() {
        Ok(ast) => {
            println!("{:#?}", ast);

            let mut visitor = Visitor::new();
            
            for statement in ast {
                match visitor.visit_statement(&statement) {
                    Ok(()) => (),
                    Err(e) => e.display(Some(&source.lines().collect()))
                }
            }
        }
        Err(e) => e.display(Some(&source.lines().collect()))
    }
}
