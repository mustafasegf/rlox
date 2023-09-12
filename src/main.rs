use std::{collections::HashMap, env};

use itertools::multipeek;

#[derive(Debug, Clone)]
enum TokenType {
    LeftParent,
    RightParen,
    LeftBrace,
    RightBrace,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BangEqual,
    EQUAL,
    EqualEqual,
    GREATER,
    GreaterEqual,
    LESS,
    LessEqual,

    // Literals.
    IDENTIFIER(String),
    STRING(String),
    NUMBER(f64),

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

#[derive(Debug)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    line: i32,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, line: i32) -> Token {
        Token {
            token_type,
            lexeme,
            line,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => run_prompt(),
        2 => {
            if let Err(e) = run_file(&args[1]) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        _ => println!("More than one argument given"),
    }
}

fn run_file(file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let bytes =
        std::fs::read_to_string(file_name).map_err(|e| format!("Error reading file: {}", e))?;

    println!("reading file: {}", file_name);
    // println!("file contents: {}", bytes);

    let tokens = scan_tokens(&bytes);
    println!("tokens: {:#?}", tokens);

    Ok(())
}

fn run_prompt() {
    let mut buffer = String::new();
    loop {
        println!("> ");
        if let Err(..) = std::io::stdin().read_line(&mut buffer) {
            continue;
        }

        buffer = buffer.trim().to_string();

        if buffer == "" {
            break;
        }

        println!("You typed: {}", buffer);
        buffer.clear();
    }
}

fn scan_tokens(buffer: &str) -> Vec<Token> {
    // todo, make current start increment when iterating

    let keywords = HashMap::from([
        ("and", TokenType::AND),
        ("class", TokenType::CLASS),
        ("else", TokenType::ELSE),
        ("false", TokenType::FALSE),
        ("for", TokenType::FOR),
        ("fun", TokenType::FUN),
        ("if", TokenType::IF),
        ("nil", TokenType::NIL),
        ("or", TokenType::OR),
        ("print", TokenType::PRINT),
        ("return", TokenType::RETURN),
        ("super", TokenType::SUPER),
        ("this", TokenType::THIS),
        ("true", TokenType::TRUE),
        ("var", TokenType::VAR),
        ("while", TokenType::WHILE),
    ]);

    let mut start = 0;
    let mut current = 0;
    let mut line = 1;

    let mut tokens: Vec<Token> = Vec::new();

    // let mut iter = buffer.chars().peekable();
    let mut iter = multipeek(buffer.chars());
    while let Some(c) = iter.next() {
        start = current;

        // logic
        match c {
            '(' => tokens.push(Token::new(TokenType::LeftParent, c.to_string(), line)),
            ')' => tokens.push(Token::new(TokenType::RightParen, c.to_string(), line)),
            '{' => tokens.push(Token::new(TokenType::LeftBrace, c.to_string(), line)),
            '}' => tokens.push(Token::new(TokenType::RightBrace, c.to_string(), line)),
            ',' => tokens.push(Token::new(TokenType::COMMA, c.to_string(), line)),
            '.' => tokens.push(Token::new(TokenType::DOT, c.to_string(), line)),
            '-' => tokens.push(Token::new(TokenType::MINUS, c.to_string(), line)),
            '+' => tokens.push(Token::new(TokenType::PLUS, c.to_string(), line)),
            ';' => tokens.push(Token::new(TokenType::SEMICOLON, c.to_string(), line)),
            '*' => tokens.push(Token::new(TokenType::STAR, c.to_string(), line)),
            '!' => {
                if iter.peek() == Some(&'=') {
                    let chars = format!("{c}{}", iter.next().unwrap_or_default());
                    tokens.push(Token::new(TokenType::BangEqual, chars, line));
                } else {
                    tokens.push(Token::new(TokenType::BANG, c.to_string(), line));
                }
            }
            '=' => {
                if iter.peek() == Some(&'=') {
                    let chars = format!("{c}{}", iter.next().unwrap_or_default());
                    tokens.push(Token::new(TokenType::EqualEqual, chars, line));
                } else {
                    tokens.push(Token::new(TokenType::EQUAL, c.to_string(), line));
                }
            }
            '<' => {
                if iter.peek() == Some(&'=') {
                    let chars = format!("{c}{}", iter.next().unwrap_or_default());
                    tokens.push(Token::new(TokenType::LessEqual, chars, line));
                } else {
                    tokens.push(Token::new(TokenType::LESS, c.to_string(), line));
                }
            }
            '>' => {
                if iter.peek() == Some(&'=') {
                    let chars = format!("{c}{}", iter.next().unwrap_or_default());
                    tokens.push(Token::new(TokenType::GreaterEqual, chars, line));
                } else {
                    tokens.push(Token::new(TokenType::GREATER, c.to_string(), line));
                }
            }
            '/' => {
                if iter.peek() == Some(&'/') {
                    // ini komentar, skip sampe end of line
                    while let Some(c) = iter.next() {
                        if c == '\n' {
                            line += 1;
                            break;
                        }
                    }
                } else {
                    tokens.push(Token::new(TokenType::SLASH, c.to_string(), line));
                }
            }

            ' ' | '\r' | '\t' => (),
            '"' => {
                let mut chars = vec![];
                let ext = loop {
                    match iter.next() {
                        Some(c) if c == '"' => break true,
                        Some(c) => chars.push(c),
                        None => break false,
                    }
                };

                if !ext {
                    error(line, "Unterminated string");
                } else {
                    let lexeme = chars.iter().collect::<String>();
                    tokens.push(Token::new(TokenType::STRING(lexeme.clone()), lexeme, line));
                }
            }
            c if c.is_ascii_digit() => {
                let mut digits = vec![c];
                let ext = loop {
                    let c = iter.peek().cloned();
                    match c {
                        Some(c) if c.is_ascii_digit() => digits.push(iter.next().unwrap()),
                        Some(c) => {
                            if c == '.' && iter.peek().unwrap_or(&'a').is_ascii_digit() {
                                digits.push(iter.next().unwrap());
                                digits.push(iter.next().unwrap());
                            } else {
                                break true;
                            }
                        }
                        None => break false,
                    }
                };

                if !ext {
                    error(line, "Unterminated number");
                } else {
                    let lexeme = digits.iter().collect::<String>();
                    tokens.push(Token::new(
                        TokenType::NUMBER(lexeme.parse::<f64>().unwrap()),
                        lexeme,
                        line,
                    ));
                }
            }
            c if c.is_ascii_alphabetic() => {
                let mut chars = vec![c];
                let ext = loop {
                    let c = iter.peek().cloned();
                    match c {
                        Some(c) if c.is_ascii_alphanumeric() => chars.push(iter.next().unwrap()),
                        Some(_) => break true,
                        None => break false,
                    }
                };

                if !ext {
                    error(line, "Unterminated identifier");
                } else {
                    let lexeme = chars.iter().collect::<String>();
                    let token_type = keywords
                        .get(lexeme.as_str())
                        .map(|t| t.clone())
                        .unwrap_or(TokenType::IDENTIFIER(lexeme.clone()));

                    tokens.push(Token::new(token_type, lexeme, line));
                }
            }
            '\n' => line += 1,

            _ => error(line, format!("Unknown character: {}", c).as_str()),
        }
    }

    tokens.push(Token::new(TokenType::EOF, "".to_string(), line));

    tokens
}
fn error(line: i32, message: &str) {
    report(line, "", message);
}

fn report(line: i32, whr: &str, message: &str) {
    eprintln!("[line {}] Error {}: {}", line, whr, message);
}
