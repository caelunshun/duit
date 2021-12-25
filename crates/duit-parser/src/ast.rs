use crate::lexer::SyntaxKind;

use num_traits::FromPrimitive;

pub type SyntaxNode = rowan::SyntaxNode<Lang>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        SyntaxKind::from_u16(raw.0).unwrap()
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}

macro_rules! ast_node {
    ($node:ident) => {
        pub struct $node {
            syntax: SyntaxNode,
        }

        impl $node {
            pub fn cast(syntax: SyntaxNode) -> Option<Self> {
                match syntax.kind() {
                    SyntaxKind::$node => Some(Self { syntax }),
                    _ => None,
                }
            }

            pub fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }
    };
}

macro_rules! ast_nodes {
    ($($node:ident,)+) => {
        $(
            ast_node!($node);
        )*
    }
}

ast_nodes! {
    Root,

    DefEnum,
    DefStruct,
    DefStructField,

    DefComponent,
    DefProperty,
    DefCallback,

    ConditionalElements,
    Element,
    SetProperty,

    Expr,
    ExprAssign,
    ExprLitEnum,
    ExprClosure,

    Type,

    Ident,
}

impl DefEnum {
    pub fn is_exported(&self) -> bool {
        self.syntax
            .children()
            .any(|c| c.kind() == SyntaxKind::KwExport)
    }

    pub fn ident(&self) -> Option<Ident> {
        self.syntax.children().find_map(Ident::cast)
    }

    pub fn variants(&self) -> impl Iterator<Item = Ident> + '_ {
        self.syntax.children().filter_map(Ident::cast).skip(1)
    }
}

impl DefStruct {
    pub fn is_exported(&self) -> bool {
        self.syntax
            .children()
            .any(|c| c.kind() == SyntaxKind::KwExport)
    }

    pub fn ident(&self) -> Option<Ident> {
        self.syntax.children().find_map(Ident::cast)
    }

    pub fn fields(&self) -> impl Iterator<Item = DefStructField> + '_ {
        self.syntax.children().filter_map(DefStructField::cast)
    }
}

impl DefStructField {
    pub fn ident(&self) -> Option<Ident> {
        self.syntax.children().find_map(Ident::cast)
    }

    pub fn typ(&self) -> Option<Type> {
        self.syntax.children().find_map(Type::cast)
    }
}

pub enum Typ {
    Int,
    Float,
    Ident(Ident),
}

impl Type {
    pub fn get(&self) -> Option<Typ> {
        self.syntax.children().find_map(|s| match s.kind() {
            SyntaxKind::KwInt => Some(Typ::Int),
            SyntaxKind::KwFloat => Some(Typ::Float),
            SyntaxKind::Ident => Some(Typ::Ident(Ident::cast(s).unwrap())),
            _ => None,
        })
    }
}

impl DefComponent {
    pub fn is_exported(&self) -> bool {
        self.syntax
            .children()
            .any(|c| c.kind() == SyntaxKind::KwExport)
    }

    pub fn ident(&self) -> Option<Ident> {
        self.syntax.children().find_map(Ident::cast)
    }

    pub fn properties(&self) -> impl Iterator<Item = DefProperty> + '_ {
        self.syntax.children().filter_map(DefProperty::cast)
    }

    pub fn callbacks(&self) -> impl Iterator<Item = DefCallback> + '_ {
        self.syntax.children().filter_map(DefCallback::cast)
    }

    pub fn elements(&self) -> impl Iterator<Item = Element> + '_ {
        self.syntax.children().filter_map(Element::cast)
    }

    pub fn conditional_elements(&self) -> impl Iterator<Item = ConditionalElements> + '_ {
        self.syntax.children().filter_map(ConditionalElements::cast)
    }
}

impl ConditionalElements {
    pub fn condition(&self) -> Option<Expr> {
        self.syntax.children().find_map(Expr::cast)
    }

    pub fn elements(&self) -> impl Iterator<Item=Element> + '_ {
        self.syntax.children().filter_map(Element::cast)
    }
}

impl Element {
    pub fn ident(&self) -> Option<Ident> {
        self.syntax.children().find_map(Ident::cast)
    }

    pub fn set_properties(&self) -> impl Iterator<Item=SetProperty> + '_ {
        self.syntax.children().filter_map(SetProperty::cast)
    }

    pub fn 
}
