use logos::Logos;
use num_derive::FromPrimitive;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, FromPrimitive, Logos)]
pub enum SyntaxKind {
    #[token("component")]
    KwComponent,
    #[token("property")]
    KwProperty,
    #[token("callback")]
    KwCallback,
    #[token("enum")]
    KwEnum,
    #[token("struct")]
    KwStruct,
    #[token("export")]
    KwExport,
    #[token("emit")]
    KwEmit,
    #[token("bool")]
    KwBool,
    #[token("int")]
    KwInt,
    #[token("float")]
    KwFloat,
    #[token("if")]
    KwIf,
    #[token("for")]
    KwFor,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("=")]
    Assign,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token("|")]
    Pipe,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    #[regex("\"[^\"]*\"")]
    LitStr,

    #[regex("[ \t\n\r]+")]
    Whitespace,

    // AST nodes
    Root,
    DefEnum,
    DefStruct,
    DefStructField,
    DefComponent,
    DefProperty,
    DefCallback,
    ConditionalElements,
    Element,
    SetField,
    ExprClosure,

    Expr,
    ExprAssign,
    ExprLitEnum,

    Type,

    #[error]
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_example() {
        let s = include_str!("../../../examples/menu.duit");
        let mut lexer = SyntaxKind::lexer(s);

        while let Some(tok) = lexer.next() {
            if tok != SyntaxKind::Whitespace {
                println!("{:?} \"{}\"", tok, lexer.slice());
            }
            assert_ne!(tok, SyntaxKind::Error);
        }
    }
}
