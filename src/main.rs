use std::collections::HashMap;

#[derive(Debug, Clone)]
/// # Token
/// Type that a token can be
/// ## Vector
/// This is a list that contains tokens.
/// ### Example
/// ```txt
/// (list 1 2)
/// ```
/// The parenthesis start a Vector
/// ## Symbol
/// Anything that is not a Vector
/// ### Example
/// ```txt
/// 1
/// 2
/// Bruh
/// "adsf" "a"
/// ```
/// Symbols are split by white space, unless there is a delimiter between them.
/// Currently only "" are a recognized delimiter.
enum Token {
    Vector(Vec<Token>),
    Symbol(String),
}

/// # Helper method for tokenizer
/// Takes the token on the top of the stack, which is a Token::Vector
/// and places new_tk into that Token::Vector at the end
///
/// # SAFETY
/// Make sure that `*mut Token'`s in stack are nested in each other
/// to prevent dangling pointers (UB)
/// ## Example
/// ```txt
/// stack = vec![1, vec![2]]
///         ^        ^
///         |        |
///         +- ptr1  +- ptr2
/// ```
/// ptr1 must be below ptr2 in the stack to prevent dangling pointers
/// caused by modifying the top of the stack only
///
unsafe fn push_token(
    new_tk: Token,
    stack: &mut Vec<*mut Token>,
    f: Option<fn(&mut Vec<Token>, &mut Vec<*mut Token>)>,
) {
    let last = *stack.last().unwrap();
    unsafe {
        let tk = &mut (*last);
        match tk {
            Token::Vector(x) => {
                x.push(new_tk);
                if let Some(fnc) = f {
                    fnc(x, stack);
                }
            }
            _ => panic!("Expected stack to only have Token::Vector"),
        }
    }
}

/// Convert string into ast of tokens.
fn tokenizer(ast_text: &str) -> Token {
    let mut root = Token::Vector(vec![Token::Symbol("do".to_string())]);
    let mut stack = Vec::new();
    let mut cur_symbol = "".to_string();
    let delimiters = "\"\'";
    let mut delimiter_stack = Vec::new();
    stack.push(&mut root as *mut Token);
    for c in ast_text.chars() {
        match c {
            '(' => {
                let new_list = Vec::new();
                unsafe {
                    push_token(
                        Token::Vector(new_list),
                        &mut stack,
                        Some(|x: &mut Vec<Token>, s: &mut Vec<*mut Token>| {
                            s.push(x.last_mut().unwrap() as *mut Token);
                        }),
                    )
                }
            }
            ')' => {
                if !delimiter_stack.is_empty() {
                    panic!("All non list delimiters should be complete");
                }
                if cur_symbol != "" {
                    unsafe { push_token(Token::Symbol(cur_symbol), &mut stack, None) };
                    cur_symbol = "".to_string();
                }
                stack.pop();
            }
            ' ' => {
                if !delimiter_stack.is_empty() {
                    cur_symbol += &c.to_string();
                } else if cur_symbol != "" {
                    unsafe { push_token(Token::Symbol(cur_symbol), &mut stack, None) };
                    cur_symbol = "".to_string();
                }
            }
            '\n' => {
                if cur_symbol != "" {
                    unsafe { push_token(Token::Symbol(cur_symbol), &mut stack, None) };
                    cur_symbol = "".to_string();
                }
            }
            _ => {
                if delimiters.contains(c) {
                    delimiter_stack.push(c);
                    // [', '] -> []
                    // [", ', '] -> ["]
                    if delimiter_stack.len() > 1
                        && delimiter_stack[delimiter_stack.len() - 1]
                            == delimiter_stack[delimiter_stack.len() - 2]
                    {
                        delimiter_stack.pop();
                        delimiter_stack.pop();
                    }
                }
                cur_symbol += &c.to_string();
            }
        }
    }
    return root;
}

#[derive(Debug, Clone)]
/// # DLisp Types
/// These are the types that a symbol can be
enum LType {
    Number(f64),
    String(String),
    Fun(Fun),
    Token(Token),
    Nil,
}

#[derive(Debug, Clone)]
/// # Lisp function
/// ## Native
/// pointer to function written in rust
/// ## Lisp
/// Token is the root of tree in this format
/// ```txt
/// (defun <fname> (<fargs>) <function body>)
/// ```
///
/// TODO add docstrings attached to functions
enum Fun {
    Native(fn(Vec<LType>, &mut HashMap<String, LType>) -> LType),
    Lisp(Token),
}

fn eval_args(root: LType, symbols: &mut HashMap<String, LType>) -> LType {
    let root = match root {
        LType::Token(token) => token,
        LType::String(s) => return symbols.get(&s).unwrap_or(&LType::String(s)).clone(),
        _ => return root,
    };
    match root {
        Token::Vector(_) => return eval_ast(&root, symbols),
        Token::Symbol(x) => return symbols.get(&x).unwrap_or(&LType::String(x)).clone(),
    }
}

/// Creates a symbol list for evaluating an ast
/// This defines multiple rust functions to be called in lisp
/// and multiple lisp functions
///
/// TODO refactor the rust functions from lambdas -> regular functions
/// TODO convert rust functions into macros if they need to supress evaluation? instead of forcing every rust function to evaluate its args manually
fn stdlib() -> HashMap<String, LType> {
    let mut stdlib = HashMap::new();
    stdlib.insert(
        "print".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s))
                .collect::<Vec<_>>();
            println!("{:?}", &v[1..v.len()]);
            return LType::Nil;
        })),
    );
    stdlib.insert(
        "=".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s))
                .collect::<Vec<_>>();
            let args = v[1..v.len()].iter().map(|x| match x {
                LType::Number(x) => Some(*x),
                LType::String(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                LType::Token(x) => match x {
                    Token::Symbol(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                    _ => None,
                },
                _ => None,
            });
            if args.clone().filter(|x| x.is_none()).count() > 0 {
                return LType::Nil;
            }
            let mut result = 1;
            let mut first = None;
            let args_len = args.len();
            args.map(|x| x.unwrap()).for_each(|x| {
                if first.is_none() {
                    first = Some(x);
                } else if let Some(first) = first
                    && first == x
                {
                    result += 1;
                } else {
                    result += 0;
                }
            });
            let mut out = 0.0;
            if args_len == result {
                out = 1.0;
            }
            return LType::Number(out);
        })),
    );
    stdlib.insert(
        "+".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s))
                .collect::<Vec<_>>();
            let v = &v[1..v.len()];
            let args = v.iter().map(|x| match x {
                LType::Number(x) => Some(*x),
                LType::String(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                LType::Token(x) => match x {
                    Token::Symbol(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                    _ => None,
                },
                _ => None,
            });
            if args.clone().filter(|x| x.is_none()).count() > 0 {
                return LType::Nil;
            }
            let result = args.map(|x| x.unwrap()).sum::<f64>();
            return LType::Number(result);
        })),
    );
    stdlib.insert(
        "*".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            // copied from +
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s))
                .collect::<Vec<_>>();
            let v = &v[1..v.len()];
            let args = v.iter().map(|x| match x {
                LType::Number(x) => Some(*x),
                LType::String(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                LType::Token(x) => match x {
                    Token::Symbol(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                    _ => None,
                },
                _ => None,
            });
            if args.clone().filter(|x| x.is_none()).count() > 0 {
                return LType::Nil;
            }
            let result = args.map(|x| x.unwrap()).reduce(|acc, e| acc * e).unwrap();
            return LType::Number(result);
        })),
    );
    stdlib.insert(
        "/".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            // copied from +
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s))
                .collect::<Vec<_>>();
            let v = &v[1..v.len()];
            let args = v.iter().map(|x| match x {
                LType::Number(x) => Some(*x),
                LType::String(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                LType::Token(x) => match x {
                    Token::Symbol(x) => x.parse::<f64>().map_or(None, |x| Some(x)),
                    _ => None,
                },
                _ => None,
            });
            if args.clone().filter(|x| x.is_none()).count() > 0 {
                return LType::Nil;
            }
            let result = args.map(|x| x.unwrap()).reduce(|acc, e| acc / e).unwrap();
            return LType::Number(result);
        })),
    );
    stdlib.insert(
        "do".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s))
                .collect::<Vec<_>>();
            return v.last().unwrap().clone();
        })),
    );
    stdlib.insert(
        "if".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            // let v = v
            //     .iter()
            //     .map(|x| eval_args(x.clone(), s))
            //     .collect::<Vec<_>>();
            let cond = eval_args(v[1].clone(), s);
            let cond = match cond {
                LType::Number(x) => Some(x),
                LType::String(x) => x.parse::<f64>().map(|x| Some(x)).unwrap_or(None),
                LType::Token(x) => match x {
                    Token::Symbol(x) => x.parse::<f64>().map(|x| Some(x)).unwrap_or(None),
                    _ => None,
                },
                _ => None,
            };
            if cond.is_none() {
                panic!("Missing condition in if statement")
            }
            let cond = cond.unwrap();
            if cond == 1.0 {
                return eval_args(v.get(2).expect("missing arm of if statement").clone(), s);
            } else if cond == 0.0 {
                return eval_args(v.get(3).expect("missing arm of if statement").clone(), s);
            } else {
                panic!("Condition is not boolean in if statement")
            }
        })),
    );
    stdlib.insert(
        "fn".to_string(),
        LType::Fun(Fun::Native(|v, _s| {
            let mut v = v
                .iter()
                .map(|x| match x {
                    LType::Token(token) => token.clone(),
                    LType::String(x) => Token::Symbol(x.clone()),
                    _ => panic!("{:?} not possible", x),
                })
                .collect::<Vec<_>>();
            v.insert(1, Token::Symbol("anonymous function".to_string()));
            return LType::Fun(Fun::Lisp(Token::Vector(v)));
        })),
    );
    stdlib.insert(
        "defn".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| match x {
                    LType::Token(token) => token.clone(),
                    LType::String(x) => Token::Symbol(x.clone()),
                    _ => panic!("{:?} not possible", x),
                })
                .collect::<Vec<_>>();
            let fname = match &v[1] {
                Token::Symbol(x) => x,
                _ => panic!("function name must be a string"),
            };
            s.insert(fname.to_string(), LType::Fun(Fun::Lisp(Token::Vector(v))));
            return LType::Nil;
        })),
    );
    eval_ast(
        &tokenizer(
            "
(defn - (a b) (+ a (* b -1)))
",
        ),
        &mut stdlib,
    );
    return stdlib;
}

/// Evaluate ast
/// root must be a Token::Vector or else it will panic
/// You cannot evaluate a symbol
fn eval_ast(root: &Token, symbols: &mut HashMap<String, LType>) -> LType {
    let tks = match root {
        Token::Vector(tks) => tks,
        _ => panic!("process ast expects root to be vector"),
    };

    let f = match tks.first().unwrap() {
        Token::Symbol(first_token) => match symbols
            .get(first_token)
            .expect(&format!("{} is not a recognized function", first_token))
        {
            LType::Fun(x) => x.clone(),
            _ => panic!("Not a function {:?}", first_token),
        },
        Token::Vector(tks) => match eval_ast(&Token::Vector(tks.to_vec()), symbols) {
            LType::Fun(x) => x,
            _ => panic!("Not a function {:?}", tks),
        },
    };

    let supress_evaluation = match f {
        Fun::Native(_) => true,
        Fun::Lisp(_) => false,
    };

    let mut args = Vec::new();
    for tk in tks {
        match tk {
            Token::Vector(_) => {
                if !supress_evaluation {
                    args.push(eval_ast(&tk, symbols))
                } else {
                    args.push(LType::Token(tk.clone()))
                }
            }
            Token::Symbol(x) => {
                if supress_evaluation || x.contains("\"") || x.contains("\'") {
                    args.push(LType::String(x.clone()))
                } else if let Ok(x) = x.parse::<f64>() {
                    args.push(LType::Number(x))
                } else {
                    let s = symbols
                        .get(x)
                        .expect(&format!("could not find symbol called {:?}", x));
                    args.push(s.clone());
                }
            }
        }
    }

    let result = match f {
        Fun::Native(x) => x(args.to_vec(), symbols),
        Fun::Lisp(token) => {
            let root = match token {
                Token::Vector(x) => x,
                _ => panic!("unexpected"),
            };
            let mut new_symbols = symbols.clone();
            match &root[2] {
                Token::Vector(x) => x.iter().map(|x| match x {
                    Token::Symbol(x) => x,
                    _ => panic!("unexpected vector in arglist"),
                }),
                _ => panic!("unexpected symbol instead of arglist"),
            }
            .zip(&args[1..args.len()])
            .for_each(|(s, e)| {
                new_symbols.insert(s.clone(), e.clone());
            });
            let root = &root[3];
            // println!("{:?}", root);
            return eval_ast(root, &mut new_symbols);
        }
    };

    return result;
}

fn main() {
    // let tokens = tokenizer("(print 'hi gaming')");
    // let mut heap = stdlib();
    // eval_ast(&tokens, &mut heap);

    let _tokens = tokenizer(
        "
(defn p (a) (print a))
(p \"b\")
(p (- 10 1))
(defn sub (a) (- a 10))
(p (sub 11))
(if 0 2 3)
",
    );

    let _tokens = tokenizer(
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
    let mut heap = stdlib();
    println!("{:?}", eval_ast(&_tokens, &mut heap));
}
