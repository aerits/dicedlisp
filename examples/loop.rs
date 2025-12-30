fn main() {
    let tokens = dlisp::tokenizer(
        "
(comment Loops in dlisp are basically loops in clojure)

(print 'Simple Loop')
(loop (i 10
       j (+ i 5))
   (print i j)
   (if (= i 0)
       j
       (recur (- i 1) (+ j 1))))

(print 'Long Loop')
(loop (i 0)
   (loop (j 0)
      (print i j)
      (if (= j 10)
        0
        (recur (+ j 1))))
   (if (= i 10)
       0
       (recur (+ i 1))))
",
    );
    println!(
        "{:?}",
        dlisp::eval_ast(&tokens, &mut dlisp::stdlib::stdlib())
    );
}
