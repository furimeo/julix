#[derive(Debug, Clone, PartialEq)]
pub enum FStrPart {
    Literal(String),
    Expr(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    FStr(Vec<FStrPart>),
    Ident(String),
    Print,
    PrintLn,
    Let,
    Const,
    If,
    Elif,
    Else,
    True,
    False,
    While,
    For,
    In,
    Break,
    Continue,
    Dot,
    Range,
    Function,
    Return,
    Type,
    Colon,
    Comma,
    Try,
    Catch,
    Throw,
    Question,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    EqEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    AndAnd,
    OrOr,
    Bang,
    And,
    Or,
    Not,
    Semicolon,
    Newline,
    Eof,
}

pub fn lex(src: &str) -> Vec<Token> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();

    while i < chars.len() {
        let c = chars[i];

        if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
            i += 1;
            continue;
        }

        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        if c.is_ascii_digit() {
            let start = i;
            if c == '0' && i + 1 < chars.len() && (chars[i + 1] == 'x' || chars[i + 1] == 'X') {
                i += 2;
                while i < chars.len() && (chars[i].is_ascii_hexdigit() || chars[i] == '_') {
                    i += 1;
                }
                let hex: String = chars[start + 2..i]
                    .iter()
                    .filter(|ch| **ch != '_')
                    .collect();
                let n = i64::from_str_radix(&hex, 16).unwrap_or(0);
                out.push(Token::Int(n));
            } else if c == '0'
                && i + 1 < chars.len()
                && (chars[i + 1] == 'o' || chars[i + 1] == 'O')
            {
                i += 2;
                while i < chars.len()
                    && (chars[i].is_ascii_digit() && chars[i] < '8' || chars[i] == '_')
                {
                    i += 1;
                }
                let oct: String = chars[start + 2..i]
                    .iter()
                    .filter(|ch| **ch != '_')
                    .collect();
                let n = i64::from_str_radix(&oct, 8).unwrap_or(0);
                out.push(Token::Int(n));
            } else if c == '0'
                && i + 1 < chars.len()
                && (chars[i + 1] == 'b' || chars[i + 1] == 'B')
            {
                i += 2;
                while i < chars.len() && (chars[i] == '0' || chars[i] == '1' || chars[i] == '_') {
                    i += 1;
                }
                let bin: String = chars[start + 2..i]
                    .iter()
                    .filter(|ch| **ch != '_')
                    .collect();
                let n = i64::from_str_radix(&bin, 2).unwrap_or(0);
                out.push(Token::Int(n));
            } else {
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                    i += 1;
                }
                if i < chars.len() && chars[i] == '.' && i + 1 < chars.len() && chars[i + 1] != '.'
                {
                    i += 1;
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                        i += 1;
                    }
                    let fstr: String = chars[start..i].iter().filter(|ch| **ch != '_').collect();
                    let n = fstr.parse::<f64>().unwrap_or(0.0);
                    out.push(Token::Float(n));
                } else {
                    let dec: String = chars[start..i].iter().filter(|ch| **ch != '_').collect();
                    let n = dec.parse::<i64>().unwrap_or(0);
                    out.push(Token::Int(n));
                }
            }
            continue;
        }

        if c == 'b' && i + 1 < chars.len() && chars[i + 1] == '"' {
            i += 2;
            let mut bytes = Vec::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        'n' => bytes.push(b'\n'),
                        't' => bytes.push(b'\t'),
                        'r' => bytes.push(b'\r'),
                        '\\' => bytes.push(b'\\'),
                        '"' => bytes.push(b'"'),
                        '0' => bytes.push(0),
                        'x' => {
                            let hex: String = chars[i + 1..i + 3].iter().collect();
                            i += 2;
                            bytes.push(u8::from_str_radix(&hex, 16).unwrap_or(0));
                        }
                        other => bytes.push(other as u8),
                    }
                    i += 1;
                } else {
                    let mut buf = [0u8; 4];
                    let s = chars[i].encode_utf8(&mut buf);
                    bytes.extend_from_slice(s.as_bytes());
                    i += 1;
                }
            }
            i += 1;
            out.push(Token::Bytes(bytes));
            continue;
        }

        if c == 'f' && i + 1 < chars.len() && chars[i + 1] == '"' {
            i += 2;
            let parts = lex_fstring(&chars, &mut i);
            out.push(Token::FStr(parts));
            continue;
        }

        if c == '"' {
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '0' => s.push('\0'),
                        'x' => {
                            let hex: String = chars[i + 1..i + 3].iter().collect();
                            i += 2;
                            let byte = u8::from_str_radix(&hex, 16).unwrap_or(0);
                            s.push(byte as char);
                        }
                        other => s.push(other),
                    }
                    i += 1;
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            i += 1;
            out.push(Token::Str(s));
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            match word.as_str() {
                "print" => out.push(Token::Print),
                "println" => out.push(Token::PrintLn),
                "let" => out.push(Token::Let),
                "const" => out.push(Token::Const),
                "if" => out.push(Token::If),
                "elif" => out.push(Token::Elif),
                "else" => out.push(Token::Else),
                "true" => out.push(Token::True),
                "false" => out.push(Token::False),
                "while" => out.push(Token::While),
                "for" => out.push(Token::For),
                "in" => out.push(Token::In),
                "break" => out.push(Token::Break),
                "continue" => out.push(Token::Continue),
                "function" => out.push(Token::Function),
                "return" => out.push(Token::Return),
                "type" => out.push(Token::Type),
                "try" => out.push(Token::Try),
                "catch" => out.push(Token::Catch),
                "throw" => out.push(Token::Throw),
                "and" => out.push(Token::And),
                "or" => out.push(Token::Or),
                "not" => out.push(Token::Not),
                _ => out.push(Token::Ident(word)),
            }
            continue;
        }

        match c {
            '(' => {
                out.push(Token::LParen);
                i += 1;
            }
            ')' => {
                out.push(Token::RParen);
                i += 1;
            }
            '{' => {
                out.push(Token::LBrace);
                i += 1;
            }
            '}' => {
                out.push(Token::RBrace);
                i += 1;
            }
            '[' => {
                out.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                out.push(Token::RBracket);
                i += 1;
            }
            '+' => {
                out.push(Token::Plus);
                i += 1;
            }
            '-' => {
                out.push(Token::Minus);
                i += 1;
            }
            '*' => {
                out.push(Token::Star);
                i += 1;
            }
            '/' => {
                out.push(Token::Slash);
                i += 1;
            }
            '%' => {
                out.push(Token::Percent);
                i += 1;
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Token::EqEq);
                    i += 2;
                } else {
                    out.push(Token::Assign);
                    i += 1;
                }
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Token::NotEq);
                    i += 2;
                } else {
                    out.push(Token::Bang);
                    i += 1;
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Token::LtEq);
                    i += 2;
                } else {
                    out.push(Token::Lt);
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    out.push(Token::GtEq);
                    i += 2;
                } else {
                    out.push(Token::Gt);
                    i += 1;
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    out.push(Token::AndAnd);
                    i += 2;
                } else {
                    eprintln!("lex error: unexpected char '&'");
                    std::process::exit(1);
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    out.push(Token::OrOr);
                    i += 2;
                } else {
                    eprintln!("lex error: unexpected char '|'");
                    std::process::exit(1);
                }
            }
            ';' => {
                out.push(Token::Semicolon);
                i += 1;
            }
            '?' => {
                out.push(Token::Question);
                i += 1;
            }
            ':' => {
                out.push(Token::Colon);
                i += 1;
            }
            ',' => {
                out.push(Token::Comma);
                i += 1;
            }
            '.' => {
                if i + 1 < chars.len() && chars[i + 1] == '.' {
                    out.push(Token::Range);
                    i += 2;
                } else {
                    out.push(Token::Dot);
                    i += 1;
                }
            }
            _ => {
                eprintln!("lex error: unexpected char '{}'", c);
                std::process::exit(1);
            }
        }
    }

    out.push(Token::Eof);
    out
}

fn lex_fstring(chars: &[char], i: &mut usize) -> Vec<FStrPart> {
    let mut parts = Vec::new();
    let mut literal = String::new();

    while *i < chars.len() && chars[*i] != '"' {
        if chars[*i] == '{' && *i + 1 < chars.len() && chars[*i + 1] == '{' {
            literal.push('{');
            *i += 2;
        } else if chars[*i] == '}' && *i + 1 < chars.len() && chars[*i + 1] == '}' {
            literal.push('}');
            *i += 2;
        } else if chars[*i] == '{' {
            if !literal.is_empty() {
                parts.push(FStrPart::Literal(literal.clone()));
                literal.clear();
            }
            *i += 1;
            let mut expr = String::new();
            let mut depth = 1;
            while *i < chars.len() && depth > 0 {
                if chars[*i] == '{' {
                    depth += 1;
                } else if chars[*i] == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                expr.push(chars[*i]);
                *i += 1;
            }
            *i += 1;
            parts.push(FStrPart::Expr(expr));
        } else {
            literal.push(chars[*i]);
            *i += 1;
        }
    }
    *i += 1;

    if !literal.is_empty() {
        parts.push(FStrPart::Literal(literal));
    }
    parts
}
