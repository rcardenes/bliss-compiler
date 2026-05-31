use std::{
    cmp::PartialEq,
    iter::{Iterator, Peekable},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Deref,                      // .
    Comma,                      // ,
    Semi,                       // ;
    Colon,                      // :
    Assign,                     // =
    Plus,                       // +
    Minus,                      // -
    Star,                       // *
    Slash,                      // /
    Not,                        // NOT
    Mod,                        // MOD
    And,                        // AND
    Or,                         // OR
    Xor,                        // XOR
    Eqv,                        // EQV
    Eql,                        // EQL
    Neq,                        // NEQ
    Lss,                        // LSS
    Leq,                        // LEQ
    Gtr,                        // GTR
    Geq,                        // GEQ

    LeftBrk,                    // [
    RightBrk,                   // ]
    LeftParen,                  // (
    RightParen,                 // )
    Begin,                      // BEGIN
    End,                        // END

    // Declarations
    Own,                        // OWN
    Local,                      // LOCAL
    Global,                     // GLOBAL
    External,                   // EXTERNAL
    Register,                   // REGISTER
    Bind,                       // BIND
    Structure,                  // STRUCTURE
    Map,                        // MAP

    // Expressions
    If,                         // IF
    Then,                       // THEN
    Else,                       // ELSE
    Do,                         // DO
    While,                      // WHILE
    Until,                      // UNTIL
    Incr,                       // INCR
    Decr,                       // DECR
    From,                       // FROM
    To,                         // TO
    By,                         // BY
    Case,                       // CASE
    Set,                        // SET
    Tes,                        // TES
    Select,                     // SELECT
    Nset,                       // NSET
    Otherwise,                  // OTHERWISE
    Tesn,                       // TESN

    Ident(usize),               // Ident names are stored into a separate structure.
                                // We just store an index here.

    // Literals
    IntLit(i64),
    StringLit(usize),           // String literals are stored into a separate structure.
                                // We just store an index here.
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos { line: usize, col: usize }

#[derive(Clone, Copy, Debug, Eq)]
pub struct Token {
    kind: TokenKind,
    pos: Pos,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl Token {
    pub fn at(kind: TokenKind, line: usize, col: usize) -> Self {
        Token { kind, pos: Pos { line, col } }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Char {
    c: char,
    pos: Pos,
}

impl Char {
    pub fn at(c: char, line: usize, col: usize) -> Self {
        Char { c, pos: Pos { line, col } }
    }
}

fn is_binary_digit(c: &char) -> bool {
    c.is_digit(2)
}

fn is_octal_digit(c: &char) -> bool {
    c.is_digit(8)
}

struct Lexer<I>
    where I: Iterator<Item=char>,
{
    source: Peekable<I>,
    line: usize,
    col: usize,
    last_char: Option<Char>,
    last_token: Option<Token>,
}

impl<I> Lexer<I>
    where I: Iterator<Item=char>,
{
    pub fn new(source: Peekable<I>) -> Self {
        Lexer {
            source,
            line: 1,
            col: 1,
            last_char: None,
            last_token: None,
        }
    }

    fn put_char(&mut self, c: Char) {
        self.last_char = Some(c)
    }

    pub fn put_token(&mut self, token: Token) {
        self.last_token = Some(token)
    }

    fn make_char(&mut self, c: char) -> Char {
        let (col, line) = (self.col, self.line);

        if c == '\n' {
            self.col = 1;
            self.line += 1;
        } else {
            self.col += 1;
        }

        Char{ c, pos: Pos { line, col }}
    }

    fn next(&mut self) -> Option<Char> {
        if let Some(c) = self.last_char.take() {
            Some(c)
        } else {
            self.source.next().map(|c| self.make_char(c))
        }
    }

    fn next_if_eq(&mut self, tc: char) -> Option<Char> {
        self.source.next_if_eq(&tc).map(|c| self.make_char(c))
    }

    fn next_if(&mut self, test: impl FnOnce(&char) -> bool) -> Option<Char> {
        self.source.next_if(test).map(|c| self.make_char(c))
    }

    fn skip(&mut self) -> Option<Char> {
        while let Some(ch) = self.next() {
            if !ch.c.is_whitespace() {
                return Some(ch)
            }
        }

        None
    }

    fn scan_intlit(&mut self, first: char, radix: u32, pos: Pos) -> Token {
        let test = match radix {
            2 => is_binary_digit,
            8 => is_octal_digit,
            10 => char::is_ascii_digit,
            16 => char::is_ascii_hexdigit,
            _ => panic!("Unsupported radix: {radix}")
        };
        if !test(&first) {
            panic!("Got {first} when expectinga an integer of radix {radix} at {},{}", pos.line, pos.col)
        }
        let mut value = first.to_digit(radix).unwrap() as i64;

        while let Some(Char { c, .. }) = self.next_if(test) {
            value = value * (radix as i64) + c.to_digit(radix).unwrap() as i64
        }
        Token::at(TokenKind::IntLit(value), pos.line, pos.col)
    }

    fn keyword(self, name: &str) -> Option<TokenKind> {
        match name {
            "NOT" => Some(TokenKind::Not),
            "MOD" => Some(TokenKind::Mod),
            "AND" => Some(TokenKind::And),
            "OR" => Some(TokenKind::Or),
            "XOR" => Some(TokenKind::Xor),
            "EQV" => Some(TokenKind::Eqv),
            "EQL" => Some(TokenKind::Eql),
            "NEQ" => Some(TokenKind::Neq),
            "LSS" => Some(TokenKind::Lss),
            "LEQ" => Some(TokenKind::Leq),
            "GTR" => Some(TokenKind::Gtr),
            "GEQ" => Some(TokenKind::Geq),
            "BEGIN" => Some(TokenKind::Begin),
            "END" => Some(TokenKind::End),
            "OWN" => Some(TokenKind::Own),
            "LOCAL" => Some(TokenKind::Local),
            "GLOBAL" => Some(TokenKind::Global),
            "EXTERNAL" => Some(TokenKind::External),
            "REGISTER" => Some(TokenKind::Register),
            "BIND" => Some(TokenKind::Bind),
            "STRUCTURE" => Some(TokenKind::Structure),
            "MAP" => Some(TokenKind::Map),
            "IF" => Some(TokenKind::If),
            "THEN" => Some(TokenKind::Then),
            "ELSE" => Some(TokenKind::Else),
            "DO" => Some(TokenKind::Do),
            "WHILE" => Some(TokenKind::While),
            "UNTIL" => Some(TokenKind::Until),
            "INCR" => Some(TokenKind::Incr),
            "DECR" => Some(TokenKind::Decr),
            "FROM" => Some(TokenKind::From),
            "TO" => Some(TokenKind::To),
            "BY" => Some(TokenKind::By),
            "CASE" => Some(TokenKind::Case),
            "SET" => Some(TokenKind::Set),
            "TES" => Some(TokenKind::Tes),
            "SELECT" => Some(TokenKind::Select),
            "NSET" => Some(TokenKind::Nset),
            "OTHERWISE" => Some(TokenKind::Otherwise),
            "TESN" => Some(TokenKind::Tesn),
            _ => None,
        }
    }

    pub fn scan(&mut self) -> Option<Token> {
        if let Some(t) = self.last_token.take() {
            Some(t)
        } else {
            if let Some(Char { c, pos }) = self.skip() {
                match c {
                    '.' => Some(Token { kind: TokenKind::Deref, pos }),
                    ',' => Some(Token { kind: TokenKind::Comma, pos }),
                    ';' => Some(Token { kind: TokenKind::Semi, pos }),
                    ':' => Some(Token { kind: TokenKind::Colon, pos }),
                    '=' => Some(Token { kind: TokenKind::Assign, pos }),
                    '+' => Some(Token { kind: TokenKind::Plus, pos }),
                    '-' => Some(Token { kind: TokenKind::Minus, pos }),
                    '*' => Some(Token { kind: TokenKind::Star, pos }),
                    '/' => Some(Token { kind: TokenKind::Slash, pos }),
                    '(' => Some(Token { kind: TokenKind::LeftParen, pos }),
                    ')' => Some(Token { kind: TokenKind::RightParen, pos }),
                    '[' => Some(Token { kind: TokenKind::LeftBrk, pos }),
                    ']' => Some(Token { kind: TokenKind::RightBrk, pos }),
                    '\'' => {
                        // Scan a string
                        todo!()
                    },
                    '%' => {
                        let radix = match self.next() {
                            Some(Char { c, pos }) if c == 'B' || c == 'b' => 2,
                            Some(Char { c, pos }) if c == 'O' || c == 'o' => 8,
                            Some(Char { c, pos }) if c == 'X' || c == 'x' => 16,
                            Some(Char { c, pos }) => panic!("Unexpected char '{c}' at {},{}", pos.line, pos.col),
                            None => panic!("Found EOF when expecting a numeric literal"),
                        };

                        if let Some(Char { c, pos }) = self.next() {
                            Some(self.scan_intlit(c, radix, pos))
                        } else {
                            panic!("Found EOF when expecting a numeric literal")
                        }
                    },
                    c if c.is_ascii_digit() => {
                        // Scan a number
                        Some(self.scan_intlit(c, 10, pos))
                    },
                    c if c.is_ascii_alphabetic() => {
                        // Scan an identifier or keyword
                        todo!()
                    },
                    c => {
                        panic!("Unrecognized character '{c}' at {},{}", pos.line, pos.col)
                    }
                }
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::IntoChars;
    use rstest::{rstest, fixture};

    type TestLexer = Lexer<IntoChars>;

    #[fixture]
    fn lexer(#[default("")] code: &str) -> TestLexer {
        let s = String::from(code);
        Lexer::new(s.into_chars().peekable())
    }

    #[test]
    fn test_token_at() {
        let t = Token::at(TokenKind::Semi, 3, 7);
        assert_eq!(t.kind, TokenKind::Semi);
        assert_eq!(t.pos, Pos { line: 3, col: 7 });
    }

    #[rstest]
    #[case(TokenKind::Deref, TokenKind::Deref)]
    #[case(TokenKind::Comma, TokenKind::Comma)]
    #[case(TokenKind::Semi, TokenKind::Semi)]
    #[case(TokenKind::If, TokenKind::If)]
    fn test_unit_variant_eq(#[case] a: TokenKind, #[case] b: TokenKind) {
        assert_eq!(a, b);
    }

    #[rstest]
    #[case(TokenKind::Deref, TokenKind::Comma)]
    #[case(TokenKind::Semi, TokenKind::Colon)]
    #[case(TokenKind::If, TokenKind::IntLit(0))]
    fn test_unit_variant_ne(#[case] a: TokenKind, #[case] b: TokenKind) {
        assert_ne!(a, b);
    }

    #[rstest]
    fn test_token_kind_int_lit(#[values(42, 0, i64::MAX, i64::MIN)] val: i64) {
        assert_eq!(TokenKind::IntLit(val), TokenKind::IntLit(val));
        assert_ne!(TokenKind::IntLit(val), TokenKind::IntLit(val.wrapping_add(1)));
    }

    #[test]
    fn test_token_clone() {
        let t = Token::at(TokenKind::While, 1, 2);
        assert_eq!(t.clone(), t);
    }

    #[test]
    fn test_token_equality_does_not_depend_on_position() {
        let a = Token::at(TokenKind::Star, 1, 1);
        let b = Token::at(TokenKind::Star, 1, 2);
        assert_eq!(a, b);
    }

    #[test]
    fn test_debug_format() {
        let t = Token::at(TokenKind::Plus, 0, 0);
        let d = format!("{:?}", t);
        assert!(!d.is_empty());
    }

    #[rstest]
    fn test_token_kind_ident(#[values(0, usize::MAX)] val: usize) {
        assert_eq!(TokenKind::Ident(val), TokenKind::Ident(val));
    }

    #[rstest]
    fn test_token_kind_string_lit(#[values(5, 0)] val: usize) {
        assert_eq!(TokenKind::StringLit(val), TokenKind::StringLit(val));
    }

    #[test]
    fn test_token_kind_ident_vs_string_lit() {
        assert_ne!(TokenKind::Ident(3), TokenKind::StringLit(3));
    }

    #[test]
    fn test_token_copy() {
        let a = Token::at(TokenKind::Assign, 2, 4);
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn test_pos_zero() {
        let t = Token::at(TokenKind::IntLit(0), 0, 0);
        assert_eq!(t.pos, Pos { line: 0, col: 0 });
    }

    #[test]
    fn test_put_char() {
        let mut lex = lexer("");
        assert!(lex.last_char.is_none());
        let c = Char::at('c', 1, 1);
        lex.put_char(c.clone());
        assert_eq!(lex.last_char, Some(c))
    }

    #[test]
    fn test_put_token() {
        let mut lex = lexer("");
        let t = Token { kind: TokenKind::Semi, pos: Pos { line: 1, col: 1 } };
        assert!(lex.last_token.is_none());
        lex.put_token(t);
        assert_eq!(lex.last_token, Some(t))
    }

    #[test]
    fn test_make_char() {
        let mut lex = lexer("c");
        let c = lex.make_char('c');
        assert_eq!(c, Char::at('c', 1, 1))
    }

    #[rstest]
    #[case("f", Some(Char::at('f', 1, 1)))]
    #[case("", None)]
    fn test_next(#[case] input: &str, #[case] expected: Option<Char>) {
        assert_eq!(lexer(input).next(), expected)
    }

    #[test]
    fn test_next_returns_put_back_char() {
        let mut lex = lexer("c");
        let Some(c) = lex.next() else { panic!("Should return a Char!") };
        lex.put_char(c.clone());
        assert_eq!(lex.next(), Some(c));
        assert!(lex.last_char.is_none())
    }

    #[test]
    fn test_newline_advances_line() {
        let mut lex = lexer("\n");
        assert_eq!(lex.next(), Some(Char { c: '\n', pos: Pos { line:1, col: 1 } }));
        assert_eq!(lex.line, 2);
        assert_eq!(lex.col, 1);
    }

    #[test]
    fn test_skip() {
        let mut lex = lexer(" \t\nc");
        let c = Some(Char::at('c', 2, 1));
        assert_eq!(lex.skip(), c);
        assert_eq!(lex.skip(), None)
    }

    #[rstest]
    #[case("f", 'f', Some(Char::at('f', 1, 1)))]
    #[case("f", 'k', None)]
    #[case("", 'k', None)]
    fn test_next_if_eq(#[case] input: &str, #[case] test: char, #[case] expected: Option<Char>) {
        assert_eq!(lexer(input).next_if_eq(test), expected)
    }

    #[rstest]
    #[case("NOT", Some(TokenKind::Not))]
    #[case("MOD", Some(TokenKind::Mod))]
    #[case("AND", Some(TokenKind::And))]
    #[case("OR", Some(TokenKind::Or))]
    #[case("XOR", Some(TokenKind::Xor))]
    #[case("EQV", Some(TokenKind::Eqv))]
    #[case("EQL", Some(TokenKind::Eql))]
    #[case("NEQ", Some(TokenKind::Neq))]
    #[case("LSS", Some(TokenKind::Lss))]
    #[case("LEQ", Some(TokenKind::Leq))]
    #[case("GTR", Some(TokenKind::Gtr))]
    #[case("GEQ", Some(TokenKind::Geq))]
    #[case("BEGIN", Some(TokenKind::Begin))]
    #[case("END", Some(TokenKind::End))]
    #[case("OWN", Some(TokenKind::Own))]
    #[case("LOCAL", Some(TokenKind::Local))]
    #[case("GLOBAL", Some(TokenKind::Global))]
    #[case("EXTERNAL", Some(TokenKind::External))]
    #[case("REGISTER", Some(TokenKind::Register))]
    #[case("BIND", Some(TokenKind::Bind))]
    #[case("STRUCTURE", Some(TokenKind::Structure))]
    #[case("MAP", Some(TokenKind::Map))]
    #[case("IF", Some(TokenKind::If))]
    #[case("THEN", Some(TokenKind::Then))]
    #[case("ELSE", Some(TokenKind::Else))]
    #[case("DO", Some(TokenKind::Do))]
    #[case("WHILE", Some(TokenKind::While))]
    #[case("UNTIL", Some(TokenKind::Until))]
    #[case("INCR", Some(TokenKind::Incr))]
    #[case("DECR", Some(TokenKind::Decr))]
    #[case("FROM", Some(TokenKind::From))]
    #[case("TO", Some(TokenKind::To))]
    #[case("BY", Some(TokenKind::By))]
    #[case("CASE", Some(TokenKind::Case))]
    #[case("SET", Some(TokenKind::Set))]
    #[case("TES", Some(TokenKind::Tes))]
    #[case("SELECT", Some(TokenKind::Select))]
    #[case("NSET", Some(TokenKind::Nset))]
    #[case("OTHERWISE", Some(TokenKind::Otherwise))]
    #[case("TESN", Some(TokenKind::Tesn))]
    #[case("FOOBAR", None)]
    fn test_keyword(#[case] input: &str, #[case] expected: Option<TokenKind>) {
        assert_eq!(lexer("").keyword(input), expected)
    }

    #[rstest]
    #[case(".", TokenKind::Deref)]
    #[case(",", TokenKind::Comma)]
    #[case(";", TokenKind::Semi)]
    #[case(":", TokenKind::Colon)]
    #[case("=", TokenKind::Assign)]
    #[case("+", TokenKind::Plus)]
    #[case("-", TokenKind::Minus)]
    #[case("*", TokenKind::Star)]
    #[case("/", TokenKind::Slash)]
    #[case("(", TokenKind::LeftParen)]
    #[case(")", TokenKind::RightParen)]
    #[case("[", TokenKind::LeftBrk)]
    #[case("]", TokenKind::RightBrk)]
    #[case("123", TokenKind::IntLit(123))]
    #[case("%B1001", TokenKind::IntLit(9))]
    #[case("%O41267", TokenKind::IntLit(17079))]
    #[case("%XDECAF", TokenKind::IntLit(912559))]
    fn test_basic_tokenization(#[case] input: &str, #[case] kind: TokenKind) {
        assert_eq!(lexer(input).scan(), Some(Token::at(kind, 1, 1)));
    }

    #[test]
    fn test_scan_from_putback() {
        let mut lex = lexer("()");
        let Some(token) = lex.scan() else { panic!("Token expected!") };
        assert!(matches!(token, Token { kind: TokenKind::LeftParen, .. }));
        lex.put_token(token);
        assert_eq!(lex.scan(), Some(token));
    }

    #[rstest]
    #[case("%B201")]
    #[case("%O8777")]
    #[case("%XKDECAF")]
    #[case("%T1234")]
    #[should_panic]
    fn test_illegal_integers(#[case] input: &str) {
        lexer(input).scan();
    }

    #[test]
    #[should_panic(expected = "Unsupported radix")]
    fn test_unsupported_radix_intlit() {
        let mut lex = lexer("");
        lex.scan_intlit('1', 5, Pos { line: 1, col: 1 });
    }

    #[rstest]
    #[case("%")]
    #[case("%B")]
    #[case("%O")]
    #[case("%X")]
    #[should_panic(expected = "Found EOF when expecting a numeric literal")]
    fn test_incomplete_intlit(#[case] input: &str) {
        lexer(input).scan();
    }

    #[test]
    #[should_panic(expected = "Unrecognized character")]
    fn test_illegal_char() {
        lexer("\\").scan();
    }

    #[test]
    fn test_find_eof() {
        assert!(lexer("").scan().is_none())
    }

    #[rstest]
    #[case("foo")]
    #[case("NOT")]
    #[case("'hello'")]
    #[should_panic]
    fn test_unimplemented_branches(#[case] input: &str) {
        lexer(input).scan();
    }
}
