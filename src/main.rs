extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
outer: number = 10

fun foo -> number {
    fun foo_inner -> number {
        inner_inner: number = outer
        inner_inner
    }
    
    inner: number = outer
    inner
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
