extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
extern print: fun string -> nil

extern struct Love {
    load:   fun -> nil
    update: fun number -> nil
    draw:   fun -> nil
}

love: Love = new Love {
    load = fun -> nil {
        print "we're loaded"
    }
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
