#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Str(String),
    Ident(String),
    Print,
    PrintLn,
    Let,
    Const,
    LParen,
    RParen,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
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

        if c == ' ' || c == '\t' || c == '\r' {
            i += 1;
            continue;
        }

        if c == '\n' {
            out.push(Token::Newline);
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
                let dec: String = chars[start..i].iter().filter(|ch| **ch != '_').collect();
                let n = dec.parse::<i64>().unwrap_or(0);
                out.push(Token::Int(n));
            }
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
                out.push(Token::Assign);
                i += 1;
            }
            ';' => {
                out.push(Token::Semicolon);
                i += 1;
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
