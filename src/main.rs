extern crate colored;

mod van;
use van::*;

fn main() {    
    let source = r#"
mut foo: [number; 5] = [1, 2, 3, 4, 5,]

a: number = foo[0]

fun foofoo -> mut [number; 5] {
    return foo
}

fun barbar -> mut [number; 5] {
    mut bar := [1, 2, 3, 4, 5,]
    return bar
}
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
