fn main() {
    let tokens = dlisp::tokenizer(
        "
(loop (i 0
       func (fn ()
                (sh echo i)))
   (func)
   (recur (+ i 1)))
",
    );
    println!("{:?}", dlisp::eval_ast(&tokens, &mut dlisp::stdlib()));
}
