extern crate colored;

mod van;
use van::*;

fn main() {
    let source = r#"
    a: i32      = 10
    mut b: char = '\n'
    
    c := r"hey?"

    mut d := [1 2 3 4 5]

    fun fib n: i32 -> i128 {
      match n {
        | 0 -> 0
        | 1 -> 1
        | n -> fib (n - 1) + fib (n - 2)
      }
    }

    function fib -> i128 {
      | 0 -> 0
      | 1 -> 1
      | n -> fib (n - 1) + fib (n - 2)
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
