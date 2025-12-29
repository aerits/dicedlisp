use std::collections::HashMap;
use thiserror::Error;

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
pub enum Token {
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
pub fn tokenizer(ast_text: &str) -> Token {
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
pub enum LType {
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
pub enum Fun {
    Native(fn(Vec<LType>, &mut HashMap<String, LType>) -> Result<LType, EvaluationError>),
    Lisp(Token),
}

fn eval_args(root: LType, symbols: &mut HashMap<String, LType>) -> Result<LType, EvaluationError> {
    let root = match root {
        LType::Token(token) => token,
        LType::String(s) => return Ok(symbols.get(&s).unwrap_or(&LType::String(s)).clone()),
        _ => return Ok(root),
    };
    match root {
        Token::Vector(_) => return eval_ast(&root, symbols),
        Token::Symbol(x) => return Ok(symbols.get(&x).unwrap_or(&LType::String(x)).clone()),
    }
}

/// Creates a symbol list for evaluating an ast
///
/// This defines multiple rust functions to be called in lisp
///
/// and multiple lisp functions
///
/// TODO refactor the rust functions from lambdas -> regular functions
/// TODO convert rust functions into macros if they need to supress evaluation? instead of forcing every rust function to evaluate its args manually
pub fn stdlib() -> HashMap<String, LType> {
    let mut stdlib = HashMap::new();
    stdlib.insert(
        "print".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s).unwrap())
                .collect::<Vec<_>>();
            println!("{:?}", &v[1..v.len()]);
            return Ok(LType::Nil);
        })),
    );
    stdlib.insert(
        "=".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s).unwrap())
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
                return Ok(LType::Nil);
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
            return Ok(LType::Number(out));
        })),
    );
    stdlib.insert(
        "+".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s).unwrap())
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
                return Ok(LType::Nil);
            }
            let result = args.map(|x| x.unwrap()).sum::<f64>();
            return Ok(LType::Number(result));
        })),
    );
    stdlib.insert(
        "*".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            // copied from +
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s).unwrap())
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
                return Ok(LType::Nil);
            }
            let result = args.map(|x| x.unwrap()).reduce(|acc, e| acc * e).unwrap();
            return Ok(LType::Number(result));
        })),
    );
    stdlib.insert(
        "/".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            // copied from +
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s).unwrap())
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
                return Ok(LType::Nil);
            }
            let result = args.map(|x| x.unwrap()).reduce(|acc, e| acc / e).unwrap();
            return Ok(LType::Number(result));
        })),
    );
    stdlib.insert(
        "comment".to_string(),
        LType::Fun(Fun::Native(|_, _| {
            return Ok(LType::Nil);
        })),
    );
    stdlib.insert(
        "do".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let v = v
                .iter()
                .map(|x| eval_args(x.clone(), s))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(v.last().unwrap().clone());
        })),
    );
    stdlib.insert(
        "loop".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            let loop_binding = v[1].clone();
            let loop_binding = if let LType::Token(Token::Vector(x)) = loop_binding {
                x
            } else {
                panic!("Expected loop binding to be Token::Vector");
            };
            assert!(loop_binding.len() % 2 == 0);
            let mut s = s.clone();

            let loop_binding = loop_binding.iter().clone().enumerate();

            let even = loop_binding
                .clone()
                .filter(|(i, _)| i % 2 == 0)
                .map(|(_, x)| x);

            let args = even.clone().map(|x| match x {
                Token::Symbol(x) => x,
                _ => panic!("expected loop binding name to be symbol"),
            });

            let odd = loop_binding.filter(|(i, _)| i % 2 == 1).map(|(_, x)| x);
            let combined = even.zip(odd);
            combined.for_each(|(symbol, tk)| {
                let symbol = match symbol {
                    Token::Symbol(x) => x,
                    _ => panic!("expected loop binding name to be symbol"),
                };
                let tk = eval_args(LType::Token(tk.clone()), &mut s).unwrap();
                s.insert(symbol.clone(), tk);
            });
            loop {
                let mut has_recur = false;
                let mut new_recur: Vec<Token> = Vec::new();
                let v = &v[2..v.len()]
                    .iter()
                    .map(|x| eval_args(x.clone(), &mut s))
                    .map(|x| match x {
                        Ok(_) => x,
                        Err(ref e) => match e {
                            EvaluationError::SymbolNotRecognized(s, origin) => {
                                if s == "recur" {
                                    if has_recur {
                                        panic!("should not have more than 1 recur in a loop")
                                    }
                                    has_recur = true;
                                    new_recur = match origin {
                                        Token::Vector(x) => x.clone(),
                                        _ => panic!("impossible"),
                                    }
                                }
                                Ok(LType::Nil)
                            }
                            _ => x,
                        },
                    })
                    .map(|x| x.unwrap())
                    .collect::<Vec<_>>();
                if has_recur {
                    let args = args.clone();

                    let v = new_recur[1..new_recur.len()]
                        .iter()
                        .map(|x| eval_args(LType::Token(x.clone()), &mut s).unwrap())
                        .collect::<Vec<_>>();

                    args.zip(v).for_each(|(n, lt)| {
                        s.insert(n.clone(), lt);
                    });
                    continue;
                }
                return Ok(v.last().unwrap().clone());
            }
        })),
    );
    stdlib.insert(
        "if".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            // let v = v
            //     .iter()
            //     .map(|x| eval_args(x.clone(), s))
            //     .collect::<Vec<_>>();
            let cond = eval_args(v[1].clone(), s).unwrap();
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
            return Ok(LType::Fun(Fun::Lisp(Token::Vector(v))));
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
            return Ok(LType::Nil);
        })),
    );
    stdlib.insert(
        "sh".to_string(),
        LType::Fun(Fun::Native(|v, s| {
            assert!(v.len() >= 2);
            let command_name = match &v[1] {
                LType::String(x) => x.clone(),
                _ => panic!("expected shell command to be string"),
            };
            let v = v.clone()[2..v.len()]
                .iter()
                .map(|x| eval_args(x.clone(), s).unwrap())
                .map(|x| match x {
                    LType::String(x) => x,
                    LType::Number(x) => x.to_string(),
                    _ => panic!("expected shell command args to be string"),
                })
                .collect::<Vec<_>>();
            let out = std::process::Command::new(&command_name)
                .args(v)
                .spawn()
                .expect(&format!("failed to run {:?}", command_name))
                .wait_with_output()
                .map(|x| match String::from_utf8(x.stdout) {
                    Ok(x) => x,
                    Err(x) => x.to_string(),
                });
            let out = match out {
                Ok(x) => x,
                Err(x) => x.to_string(),
            };

            return Ok(LType::String(out));
        })),
    );
    let _ = eval_ast(
        &tokenizer(
            "
(defn - (a b) (+ a (* b -1)))
",
        ),
        &mut stdlib,
    )
    .unwrap();
    return stdlib;
}

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Symbol not recognized {0}")]
    SymbolNotRecognized(String, Token),
    #[error("Expected type to be: {expected}, actual was {:?}", actual)]
    TypeError {
        expected: String,
        actual: LType,
        origin: Token,
    },
    #[error(
        "Wrong arity given to function `{function}`,
expected args to be {:?} ({:?}), was given {:?} ({:?})",
        expected_args,
        match expected_args {
            Token::Vector(x) => x.len().to_string(),
            _ => "Error".to_string()
        },
        given_args,
        given_args.len()
    )]
    WrongArity {
        function: String,
        expected_args: Token,
        given_args: Vec<LType>,
    },
}

/// # Evaluate ast
/// root must be a Token::Vector or else it will panic
///
/// You cannot evaluate a symbol
///
/// get `symbols` from `dlisp::stdlib()`
pub fn eval_ast(
    root: &Token,
    symbols: &mut HashMap<String, LType>,
) -> Result<LType, EvaluationError> {
    let tks = match root {
        Token::Vector(tks) => tks,
        Token::Symbol(_) => {
            return Err(EvaluationError::TypeError {
                expected: "Token::Vector".to_string(),
                actual: LType::Token(root.clone()),
                origin: root.clone(),
            });
        } // _ => panic!("process ast expects root to be vector"),
    };

    let f = match tks.first().map_or(
        Err(EvaluationError::TypeError {
            expected: "Function".to_string(),
            actual: LType::Nil,
            origin: root.clone(),
        }),
        |x| Ok(x),
    )? {
        Token::Symbol(first_token) => match match symbols
            .get(first_token) {
                Some(x) => x,
                None => return Err(EvaluationError::SymbolNotRecognized(first_token.clone(), root.clone())),
            }
            // .expect(&format!("{} is not a recognized function", first_token))
        {
            LType::Fun(x) => x.clone(),
            _ => panic!("Not a function {:?}", first_token),
        },
        Token::Vector(tks) => match eval_ast(&Token::Vector(tks.to_vec()), symbols)? {
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
                    args.push(eval_ast(&tk, symbols)?)
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
            let arglist = match &root[2] {
                Token::Vector(x) => x.iter().map(|x| match x {
                    Token::Symbol(x) => x,
                    _ => panic!("unexpected vector in arglist"),
                }),
                _ => panic!("unexpected symbol instead of arglist"),
            };

            if arglist.clone().len() != (args.len() - 1) {
                let fname = root[1].clone();
                let fname = match fname {
                    Token::Symbol(x) => x,
                    _ => panic!("impossible"),
                };
                let args = args[1..args.len()].to_vec();
                return Err(EvaluationError::WrongArity {
                    function: fname,
                    expected_args: root[2].clone(),
                    given_args: args,
                });
            }

            arglist.zip(&args[1..args.len()]).for_each(|(s, e)| {
                new_symbols.insert(s.clone(), e.clone());
            });
            let mut fn_body = Vec::new();
            fn_body.push(Token::Symbol("do".to_string()));

            root[3..root.len()]
                .iter()
                .for_each(|x| fn_body.push(x.clone()));

            return eval_ast(&Token::Vector(fn_body), &mut new_symbols);
        }
    };

    return result;
}
