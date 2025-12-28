extern crate dlisp;
fn main() {
    let _tokens = dlisp::tokenizer(
        "
(defn p (a) (print a))
(defn fib (n)
  (if (= n 1) 0
    (if (= n 2) 1
      (+ (fib (- n 1)) (fib (- n 2))))))
(p (fib 1))
(p (fib 2))
(p (fib 3))
(p (fib 4))
(p (fib 5))
(p (fib 6))
(p (fib 7))
(p (fib 8))
(p (fib 9))
(p (fib 10))
",
    );
    let mut heap = dlisp::stdlib();
    println!("{:?}", dlisp::eval_ast(&_tokens, &mut heap));
}
