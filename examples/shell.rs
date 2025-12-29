fn main() {
    let tokens = dlisp::tokenizer(
        "
(defn loop (i func)
         (func)
         (sh sleep 0.1)
         (loop (+ i 1)))

(loop 0 (fn () (print i)))
",
    );
    println!("{:?}", dlisp::eval_ast(&tokens, &mut dlisp::stdlib()));
}
