use std::collections::HashMap;

use crate::*;

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
