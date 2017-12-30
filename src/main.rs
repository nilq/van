extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
struct Point {
    x: number
    y: number
}

mut outer := 10

if "hey" ++ ", world" == "hey, world" {
    print "Hey"
}

b := 10 + 10

baba: Point = {
    fun foo x: number -> Point {
        mut a: Point = new Point {
            x = x
            y = x
        }

        a.y = 100

        a
    }

    100 |> foo
}

c: number = {
    return unless false {
        10
    } else {
        20
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
