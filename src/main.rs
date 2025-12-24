enum Token {
    Vector(Vec<Token>),
    Symbol,
    Nil,
}

fn tokenizer(text: &str) -> Token {
    for c in text.chars() {
        println!("{}", c);
    }
    return Token::Nil;
}

fn main() {
    tokenizer("(println 'hi')");
}
