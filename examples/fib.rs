fn main() {
    let _tokens = dlisp::tokenizer(
        "
(defn p (a) (print a))
(defn fib (n)
  (if (= n 1) 0
    (if (= n 2) 1
      (+ (fib (- n 1)) (fib (- n 2))))))

(comment This is an example of a comment
         Below is a recursive loop that runs until 21
         TODO: replace with the loop function from stdlib)
(defn loop (i f)
   (if (= i 21) 0
       (do
         (print 'fib: ' i (f))
         (loop (+ i 1) f))))

(p (fib 1))
(loop 2 (fn () (fib i)))
",
    );
    let mut heap = dlisp::stdlib();
    println!("{:?}", dlisp::eval_ast(&_tokens, &mut heap));
}
