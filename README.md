# dicedlisp

basic lisp like programming language

very inefficient

## example code
```dlisp
(defn fib (n)
  (if (= n 1) 0
    (if (= n 2) 1
      (+ (fib (- n 1)) (fib (- n 2))))))
```
