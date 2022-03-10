use std::cell::RefCell;
use std::rc::Rc;

use acvm::FieldElement;
use noirc_errors::Span;

use crate::node_interner::{FuncId, TypeId};
use crate::{BinaryOp, BinaryOpKind, Ident, UnaryOp};

use super::stmt::RStatement;
use super::types::{StructType, Type};

#[derive(Debug)]
pub enum RExpression {
    Ident(RIdent),
    Literal(RLiteral),
    Block(RBlockExpression),
    Prefix(RPrefixExpression),
    Infix(RInfixExpression),
    Index(RIndexExpression),
    Constructor(RConstructorExpression),
    MemberAccess(RMemberAccess),
    Call(RCallExpression),
    MethodCall(RMethodCallExpression),
    Cast(RCastExpression),
    For(RForExpression),
    If(RIfExpression),
    Tuple(Vec<RExpression>),
    Error,
}

impl RExpression {
    /// Returns an empty block expression
    pub const fn empty_block() -> RExpression {
        RExpression::Block(RBlockExpression(vec![]))
    }
}

#[derive(Debug)]
pub struct RForExpression {
    pub identifier: RIdent,
    pub start_range: Box<RExpression>,
    pub end_range: Box<RExpression>,
    pub block: Box<RExpression>,
}

#[derive(Debug)]
pub struct RIdent {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RBinaryOp {
    pub span: Span,
    pub kind: BinaryOpKind,
}

impl From<BinaryOp> for RBinaryOp {
    fn from(a: BinaryOp) -> RBinaryOp {
        RBinaryOp {
            span: a.span(),
            kind: a.contents,
        }
    }
}

#[derive(Debug)]
pub enum RLiteral {
    Array(RArrayLiteral),
    Bool(bool),
    Integer(FieldElement),
    Str(String),
}

#[derive(Debug)]
pub struct RPrefixExpression {
    pub operator: UnaryOp,
    pub rhs: Box<RExpression>,
}

#[derive(Debug)]
pub struct RInfixExpression {
    pub lhs: Box<RExpression>,
    pub operator: RBinaryOp,
    pub rhs: Box<RExpression>,
}

#[derive(Debug)]
pub struct RMemberAccess {
    pub lhs: Box<RExpression>,
    // This field is not an IdentId since the rhs of a field
    // access has no corresponding definition
    pub rhs: Ident,
}

#[derive(Debug)]
pub struct RIfExpression {
    pub condition: Box<RExpression>,
    pub consequence: Box<RExpression>,
    pub alternative: Option<Box<RExpression>>,
}

#[derive(Debug)]
pub struct RCastExpression {
    pub lhs: Box<RExpression>,
    pub r#type: Type,
}

#[derive(Debug)]
pub struct RArrayLiteral {
    // Do we really expect users to create array literals with more than 2^64-1 elements?
    pub length: u128,
    pub contents: Vec<RExpression>,
}

#[derive(Debug)]
pub struct RCallExpression {
    pub func_id: FuncId,
    pub arguments: Vec<RExpression>,
}

/// These nodes are temporary, they're
/// lowered into CallExpression nodes
/// after type checking resolves the object
/// type and the method it calls.
#[derive(Debug)]
pub struct RMethodCallExpression {
    pub method: Ident,
    pub object: Box<RExpression>,
    pub arguments: Vec<RExpression>,
}

#[derive(Debug)]
pub struct RConstructorExpression {
    pub type_id: TypeId,
    pub r#type: Rc<RefCell<StructType>>,

    // NOTE: It is tempting to make this a BTreeSet to force ordering of field
    //       names (and thus remove the need to normalize them during type checking)
    //       but doing so would force the order of evaluation of field
    //       arguments to be alphabetical rather than the ordering the user
    //       included in the source code.
    pub fields: Vec<(RIdent, RExpression)>,
}

#[derive(Debug)]
pub struct RIndexExpression {
    pub collection_name: RIdent,
    pub index: Box<RExpression>,
}

#[derive(Debug)]
pub struct RBlockExpression(pub Vec<RStatement>);
