extern crate dlisp;
fn main() {
    let tokens = dlisp::tokenizer(
        "
(defn loop (i f)
   (if (= i 0)
       0
       (do
         (f)
         (bash sleep 0.1)
         (loop (+ i 1)))))
(loop 1 (fn () (sh echo a)))
",
    );
    println!("{:?}", dlisp::eval_ast(&tokens, &mut dlisp::stdlib()));
}
