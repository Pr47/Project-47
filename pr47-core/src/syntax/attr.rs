//! # Concrete syntax tree of attributes
//!
//! Attribute syntax:
//! ```text
//! global-attribute ::= '#' '!' '[' attribute-list ']'
//!
//! attribute ::= '#' '[' attribute-list ']'
//!
//! attribute-list ::= attribute-list ',' attribute-item
//!                  | attribute-item
//!
//! attribute-item ::= attribute-identifier-item
//!                  | attribute-assign-alike-item
//!                  | attribute-call-alike-item
//!
//! attribute-identifier-item ::= identifier
//!
//! attribute-assign-alike-item ::= identifier '=' attribute-value
//!
//! attribute-call-alike-item ::= identifier '(' attribute-list ')'
//!
//! attribute-value ::= identifier
//!                   | literal
//! ```

use smallvec::SmallVec;

use crate::diag::location::{SourceLoc, SourceRange};
use crate::syntax::id::Identifier;

#[cfg_attr(test, derive(Debug))]
pub struct Attribute<'a> {
    pub items: SmallVec<[AttrItem<'a>; 4]>,

    pub hash_loc: SourceLoc,
    pub exclaim_loc: SourceLoc,
    pub left_bracket_loc: SourceLoc,
    pub right_bracket_loc: SourceLoc
}

#[cfg_attr(test, derive(Debug))]
pub enum AttrItem<'a> {
    IdentifierItem(Identifier<'a>),
    AssignLikeItem(AttrAssignLikeItem<'a>),
    CallLikeItem(AttrCallLikeItem<'a>)
}

#[cfg_attr(test, derive(Debug))]
pub struct AttrAssignLikeItem<'a> {
    pub ident: Identifier<'a>,
    pub value: AttrValue<'a>,

    pub assign_loc: SourceLoc,
}

#[cfg_attr(test, derive(Debug))]
pub struct AttrCallLikeItem<'a> {
    pub ident: Identifier<'a>,
    pub args: Vec<AttrItem<'a>>,

    pub left_paren_loc: SourceLoc,
    pub right_paren_loc: SourceLoc,
}

#[cfg_attr(test, derive(Debug))]
pub struct AttrValue<'a> {
    pub inner: AttrValueInner<'a>,
    pub range: SourceRange
}

#[cfg_attr(test, derive(Debug))]
pub enum AttrValueInner<'a> {
    Identifier(Identifier<'a>),
    IntLiteral(i64),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(&'a str),
    BoolLiteral(bool)
}

impl<'a> AttrValue<'a> {
    pub fn ident_value(ident: Identifier<'a>) -> Self {
        Self {
            inner: AttrValueInner::Identifier(ident),
            range: SourceRange::unknown()
        }
    }

    pub fn int_value(value: i64, range: SourceRange) -> Self {
        Self {
            inner: AttrValueInner::IntLiteral(value),
            range
        }
    }

    pub fn float_value(value: f64, range: SourceRange) -> Self {
        Self {
            inner: AttrValueInner::FloatLiteral(value),
            range
        }
    }

    pub fn char_value(value: char, range: SourceRange) -> Self {
        Self {
            inner: AttrValueInner::CharLiteral(value),
            range
        }
    }

    pub fn string_value(value: &'a str, range: SourceRange) -> Self {
        Self {
            inner: AttrValueInner::StringLiteral(value),
            range
        }
    }

    pub fn bool_value(value: bool, range: SourceRange) -> Self {
        Self {
            inner: AttrValueInner::BoolLiteral(value),
            range
        }
    }
}
