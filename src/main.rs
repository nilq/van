mod van;
use van::*;

fn main() {
    let source = r#"
a: i32      = 10
mut b: char = '\n'
c := r"strong raw\n string"
    "#;

    let lexer = make_lexer(source.chars().collect());

    for token in lexer {
        println!("{:?}:   {:#?}", token.content, token.token_type);
    }
}
