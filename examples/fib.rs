fn main() {
    let _tokens = dlisp::tokenizer(
        "
(defn p (a) (print a))
(defn fib (n)
  (if (= n 1) 0
    (if (= n 2) 1
      (+ (fib (- n 1)) (fib (- n 2))))))

(comment This is an example of a comment
         Below is a recursive loop that runs until 21)
(loop (i 1
       fun (fn (i) (fib i) ))
   (if (= i 21) 0
       (do
         (print 'fib: ' i (fun i))
         (recur (+ i 1) fun))))
",
    );
    let mut heap = dlisp::stdlib::stdlib();
    println!("{:?}", dlisp::eval_ast(&_tokens, &mut heap));
}
