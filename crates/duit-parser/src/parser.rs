use logos::Logos;
use rowan::{GreenNodeBuilder, Language};

use crate::{
    ast::{Lang, SyntaxNode},
    lexer::SyntaxKind,
};

#[derive(Debug, Copy, Clone)]
pub struct ParseToken<'source> {
    pub tok: SyntaxKind,
    pub text: &'source str,
}

impl Default for ParseToken<'_> {
    fn default() -> Self {
        Self {
            tok: SyntaxKind::Error,
            text: "",
        }
    }
}

pub struct Parser<'source> {
    tokens: Vec<ParseToken<'source>>,
    cursor: usize,
    builder: GreenNodeBuilder<'source>,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut lexer = SyntaxKind::lexer(source);
        let mut tokens = Vec::new();

        while let Some(token) = lexer.next() {
            tokens.push(ParseToken {
                tok: token,
                text: lexer.slice(),
            });
        }

        Self {
            tokens,
            cursor: 0,
            builder: GreenNodeBuilder::new(),
        }
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(Lang::kind_to_raw(kind));
    }

    pub fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    pub fn current_token(&self) -> ParseToken<'source> {
        self.tokens.get(self.cursor).copied().unwrap_or_default()
    }

    pub fn consume(&mut self) {
        let t = self.current_token();
        self.builder.token(Lang::kind_to_raw(t.tok), t.text);
        self.cursor += 1;
    }

    pub fn consume_ws(&mut self) {
        while self.current_token().tok == SyntaxKind::Whitespace {
            self.consume();
        }
    }

    pub fn nth(&mut self, mut n: usize) -> ParseToken<'source> {
        self.consume_ws();
        let mut c = self.cursor;
        while n > 0 {
            c += 1;
            while c < self.tokens.len() && self.tokens[c].tok == SyntaxKind::Whitespace {
                c += 1;
            }
            n -= 1;
        }
        self.tokens[c]
    }

    pub fn peek(&mut self) -> ParseToken<'source> {
        self.nth(0)
    }

    pub fn consume_if(&mut self, token: SyntaxKind) -> bool {
        if self.nth(0).tok == token {
            self.consume();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, token: SyntaxKind) -> bool {
        self.consume_if(token)
    }

    pub fn is_done(&self) -> bool {
        self.cursor >= self.tokens.len()
    }

    pub fn finish(self) -> SyntaxNode {
        SyntaxNode::new_root(self.builder.finish())
    }
}
