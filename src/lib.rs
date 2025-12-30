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

pub mod stdlib;

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
