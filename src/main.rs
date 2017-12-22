extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
interface Movable {
    translate: fun float float
    move:      fun float float
}    

struct Vector {
    x: float
    y: float
}

implement Vector as Movable {
    fun translate self: mut Vector x: float y: float {
        self.x = self.x + x
        self.y = self.y + y
    }
    
    fun move self: mut Vector x: float y: float {
        self.x = x
        self.y = y
    }
}

struct Mouse {
    pos: Vector
}

implement Mouse {
    fun at x: float y: float -> Mouse {
        new {
            x = x
            y = y
        }
    }
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
