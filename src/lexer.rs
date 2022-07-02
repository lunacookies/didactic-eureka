use logos::Logos;

pub fn lex(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut lexer = TokenKind::lexer(text);

    while let Some(kind) = lexer.next() {
        let token = Token { text: lexer.slice(), kind };
        tokens.push(token);
    }

    tokens
}

#[derive(Debug)]
pub struct Token<'a> {
    pub text: &'a str,
    pub kind: TokenKind,
}

#[derive(Debug, Logos)]
pub enum TokenKind {
    #[token("fn")]
    FnKw,
    #[token("let")]
    LetKw,
    #[token("struct")]
    StructKw,
    #[token("if")]
    IfKw,
    #[token("else")]
    ElseKw,
    #[token("for")]
    ForKw,
    #[token("return")]
    ReturnKw,
    #[token("break")]
    BreakKw,
    #[token("continue")]
    ContinueKw,

    #[regex("[a-z][a-z0-9]*")]
    Ident,
    #[regex("[0-9]+")]
    Number,
    #[regex("\"[^\"]*\"")]
    String,
    #[regex("'[^']*'")]
    Char,

    #[token("=")]
    Eq,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token("!")]
    Bang,
    #[token("+")]
    Plus,
    #[token("-")]
    Dash,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("&")]
    Pretzel,
    #[token("|")]
    Pipe,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[error]
    #[regex("[ \t\n]+", logos::skip)]
    Error,
}
