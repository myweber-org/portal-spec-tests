use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    StringToken(String),
    NumberToken(f64),
    BoolToken(bool),
    NullToken,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return None;
        }

        let ch = self.input[self.position];
        match ch {
            '{' => {
                self.position += 1;
                Some(Token::LeftBrace)
            }
            '}' => {
                self.position += 1;
                Some(Token::RightBrace)
            }
            '[' => {
                self.position += 1;
                Some(Token::LeftBracket)
            }
            ']' => {
                self.position += 1;
                Some(Token::RightBracket)
            }
            ':' => {
                self.position += 1;
                Some(Token::Colon)
            }
            ',' => {
                self.position += 1;
                Some(Token::Comma)
            }
            '"' => {
                let start = self.position + 1;
                self.position += 1;
                while self.position < self.input.len() && self.input[self.position] != '"' {
                    self.position += 1;
                }
                let end = self.position;
                self.position += 1;
                let s: String = self.input[start..end].iter().collect();
                Some(Token::StringToken(s))
            }
            '0'..='9' | '-' => {
                let start = self.position;
                while self.position < self.input.len()
                    && (self.input[self.position].is_ascii_digit()
                        || self.input[self.position] == '.'
                        || self.input[self.position] == '-'
                        || self.input[self.position] == 'e'
                        || self.input[self.position] == 'E')
                {
                    self.position += 1;
                }
                let num_str: String = self.input[start..self.position].iter().collect();
                if let Ok(num) = num_str.parse::<f64>() {
                    Some(Token::NumberToken(num))
                } else {
                    None
                }
            }
            't' if self.position + 3 < self.input.len()
                && self.input[self.position..self.position + 4].iter().collect::<String>() == "true" =>
            {
                self.position += 4;
                Some(Token::BoolToken(true))
            }
            'f' if self.position + 4 < self.input.len()
                && self.input[self.position..self.position + 5].iter().collect::<String>() == "false" =>
            {
                self.position += 5;
                Some(Token::BoolToken(false))
            }
            'n' if self.position + 3 < self.input.len()
                && self.input[self.position..self.position + 4].iter().collect::<String>() == "null" =>
            {
                self.position += 4;
                Some(Token::NullToken)
            }
            _ => None,
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn parse_value(&mut self) -> Option<JsonValue> {
        if self.position >= self.tokens.len() {
            return None;
        }

        match &self.tokens[self.position] {
            Token::LeftBrace => self.parse_object(),
            Token::LeftBracket => self.parse_array(),
            Token::StringToken(s) => {
                self.position += 1;
                Some(JsonValue::String(s.clone()))
            }
            Token::NumberToken(n) => {
                self.position += 1;
                Some(JsonValue::Number(*n))
            }
            Token::BoolToken(b) => {
                self.position += 1;
                Some(JsonValue::Bool(*b))
            }
            Token::NullToken => {
                self.position += 1;
                Some(JsonValue::Null)
            }
            _ => None,
        }
    }

    fn parse_object(&mut self) -> Option<JsonValue> {
        if self.tokens[self.position] != Token::LeftBrace {
            return None;
        }
        self.position += 1;

        let mut map = HashMap::new();
        while self.position < self.tokens.len() && self.tokens[self.position] != Token::RightBrace {
            if let Token::StringToken(key) = &self.tokens[self.position] {
                self.position += 1;
                if self.position < self.tokens.len() && self.tokens[self.position] == Token::Colon {
                    self.position += 1;
                    if let Some(value) = self.parse_value() {
                        map.insert(key.clone(), value);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }

            if self.position < self.tokens.len() && self.tokens[self.position] == Token::Comma {
                self.position += 1;
            }
        }

        if self.position < self.tokens.len() && self.tokens[self.position] == Token::RightBrace {
            self.position += 1;
            Some(JsonValue::Object(map))
        } else {
            None
        }
    }

    fn parse_array(&mut self) -> Option<JsonValue> {
        if self.tokens[self.position] != Token::LeftBracket {
            return None;
        }
        self.position += 1;

        let mut arr = Vec::new();
        while self.position < self.tokens.len() && self.tokens[self.position] != Token::RightBracket {
            if let Some(value) = self.parse_value() {
                arr.push(value);
            } else {
                return None;
            }

            if self.position < self.tokens.len() && self.tokens[self.position] == Token::Comma {
                self.position += 1;
            }
        }

        if self.position < self.tokens.len() && self.tokens[self.position] == Token::RightBracket {
            self.position += 1;
            Some(JsonValue::Array(arr))
        } else {
            None
        }
    }

    fn parse(&mut self) -> Option<JsonValue> {
        self.parse_value()
    }
}

fn parse_json(input: &str) -> Option<JsonValue> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let json = r#"{"name": "test", "value": 42, "active": true}"#;
        let result = parse_json(json);
        assert!(result.is_some());
        if let Some(JsonValue::Object(map)) = result {
            assert_eq!(map.get("name"), Some(&JsonValue::String("test".to_string())));
            assert_eq!(map.get("value"), Some(&JsonValue::Number(42.0)));
            assert_eq!(map.get("active"), Some(&JsonValue::Bool(true)));
        } else {
            panic!("Expected object");
        }
    }
}