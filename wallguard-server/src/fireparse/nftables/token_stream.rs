#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Symbol(char),
    Newline,
}

#[derive(Debug)]
pub struct TokenStream {
    tokens: Vec<Token>,
    pos: usize,
}

impl TokenStream {
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    pub fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }

    pub fn next(&mut self) -> Option<&Token> {
        if self.pos < self.tokens.len() {
            let tok = &self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn expect_ident(&mut self, expected: &str) -> Result<(), String> {
        match self.next() {
            Some(Token::Ident(s)) if s == expected => Ok(()),
            Some(t) => Err(format!("Expected ident `{}`, found {:?}", expected, t)),
            None => Err(format!("Expected ident `{}`, but reached EOF", expected)),
        }
    }

    pub fn expect_symbol(&mut self, expected: char) -> Result<(), String> {
        match self.next() {
            Some(Token::Symbol(c)) if *c == expected => Ok(()),
            Some(t) => Err(format!("Expected symbol `{}`, found {:?}", expected, t)),
            None => Err(format!("Expected symbol `{}`, but reached EOF", expected)),
        }
    }

    pub fn expect_any_ident(&mut self) -> Result<String, String> {
        match self.next() {
            Some(Token::Ident(s)) => Ok(s.clone()),
            Some(t) => Err(format!("Expected any ident, found {:?}", t)),
            None => Err("Expected any ident, but reached EOF".to_string()),
        }
    }

    pub fn expect_newline(&mut self) -> Result<(), String> {
        match self.next() {
            Some(Token::Newline) => Ok(()),
            Some(t) => Err(format!("Expected newline, found {:?}", t)),
            None => Err("Expected newline, but reached EOF".to_string()),
        }
    }

    pub fn expect_any_symbol(&mut self) -> Result<char, String> {
        match self.next() {
            Some(Token::Symbol(c)) => Ok(*c),
            Some(t) => Err(format!("Expected any symbol, found {:?}", t)),
            None => Err("Expected any symbol, but reached EOF".to_string()),
        }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn rewind(&mut self, to: usize) {
        self.pos = to.min(self.tokens.len());
    }

    pub fn skip_until(&mut self, target: &Token) -> bool {
        while let Some(token) = self.next() {
            if token == target {
                return true;
            }
        }
        false
    }
}

impl From<&str> for TokenStream {
    fn from(input: &str) -> Self {
        let mut tokens = Vec::new();

        for line in input.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut chars = line.chars().peekable();
            let mut current = String::new();

            while let Some(&ch) = chars.peek() {
                match ch {
                    '{' | '}' | ';' => {
                        if !current.is_empty() {
                            tokens.push(Token::Ident(current.clone()));
                            current.clear();
                        }
                        tokens.push(Token::Symbol(ch));
                        chars.next();
                    }
                    ' ' | '\t' => {
                        if !current.is_empty() {
                            tokens.push(Token::Ident(current.clone()));
                            current.clear();
                        }
                        chars.next();
                    }
                    _ => {
                        current.push(ch);
                        chars.next();
                    }
                }
            }

            if !current.is_empty() {
                tokens.push(Token::Ident(current));
            }

            tokens.push(Token::Newline);
        }

        TokenStream { tokens, pos: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_stream_rules() {
        let input = r#"
            table ip filter {
                chain INPUT {
                    iifname != "docker0" counter packets 0 bytes 0 drop
                    ip protocol tcp accept
                }
            }
        "#;

        let stream = TokenStream::from(input);

        let expected_tokens = vec![
            Token::Ident("table".into()),
            Token::Ident("ip".into()),
            Token::Ident("filter".into()),
            Token::Symbol('{'),
            Token::Newline,
            Token::Ident("chain".into()),
            Token::Ident("INPUT".into()),
            Token::Symbol('{'),
            Token::Newline,
            Token::Ident("iifname".into()),
            Token::Ident("!=".into()),
            Token::Ident("\"docker0\"".into()),
            Token::Ident("counter".into()),
            Token::Ident("packets".into()),
            Token::Ident("0".into()),
            Token::Ident("bytes".into()),
            Token::Ident("0".into()),
            Token::Ident("drop".into()),
            Token::Newline,
            Token::Ident("ip".into()),
            Token::Ident("protocol".into()),
            Token::Ident("tcp".into()),
            Token::Ident("accept".into()),
            Token::Newline,
            Token::Symbol('}'),
            Token::Newline,
            Token::Symbol('}'),
            Token::Newline,
        ];

        assert_eq!(stream.tokens, expected_tokens);
    }
}
