#![allow(dead_code)]

#[derive(Debug)]
pub enum Statement {
    Select(Select),
}

#[derive(Debug)]
pub struct Select {
    pub columns: Vec<ColumnRef>,
    pub table: String,
    pub where_clause: Option<Expr>,
}

#[derive(Debug)]
pub enum ColumnRef {
    Star,
    Name(String),
}

#[derive(Debug)]
pub enum Expr {
    Column(String),
    Literal(Value),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum BinaryOp {
    Gt,
    Lt,
    Eq,
    And,
    Or,
}

#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Text(String),
    Null,
}
