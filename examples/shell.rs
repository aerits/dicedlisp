extern crate dlisp;
fn main() {
    let tokens = dlisp::tokenizer(
        "
(defn loop (i func)
   (if (= i 0)
       0
       (do
         (func)
         (loop (+ i 1)))))
(loop 1 (fn () (sh echo i)))
",
    );
    println!("{:?}", dlisp::eval_ast(&tokens, &mut dlisp::stdlib()));
}
