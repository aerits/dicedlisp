# dicedlisp

basic lisp like programming language

very inefficient

loosely based on clojure's syntax without brackets

## example code
```dlisp
(defn fib (n)
  (if (= n 1) 0
    (if (= n 2) 1
      (+ (fib (- n 1)) (fib (- n 2))))))
```
Fibonacci example

Check `examples/fib.rs` to run this example

```dlisp
(loop (i 0
       func (fn ()
                (sh echo i)))
   (func)
   (recur (+ i 1)))
```
Loop example with shell command

Unlike clojure, regular `defn`'s aren't wrapped in `loop`, you have to add a `loop` manually to use `recur`

Check `examples/shell.rs` to run this example

The only way to define variables is locally with `loop` or `defn` parameters. You can set a constant by making a constant function. `(defn a () "a")`
