extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
extern print: mut fun string -> nil

extern struct graphics {
    rectangle: fun string number number number number -> nil
}

extern struct love {
    load:   fun -> nil
    update: fun number -> nil
    draw:   fun -> nil

    graphics: graphics
}

love_ := new love {
    draw = fun -> nil {
        love.graphics.rectangle "fill" 100 100 100 100

        while "hey: " ++ (1 + 1 == 2) {
            print "hey"
        }
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
