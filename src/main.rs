use std::io::{self, Write};

fn main() {
    println!("mini-sqlite v0.1");

    loop {
        print!("db > ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        if input.trim() == ".exit" {
            break;
        }

        execute_sql(&input);
    }
}

fn execute_sql(sql: &str) {
    // println!("{}", sql.to_string());
    let ast = parse(sql);
    println!("{:?}", ast);
    // let bytecode = plan(ast);
    // run(bytecode);
}

pub fn parse(sql: &str) -> Result<Statement, String> {
    let tokens = tokenize(sql);
    if tokens.is_empty() {
        return Err("Empty SQL query".to_string());
    }

    let mut i = 0;
    if tokens[0].eq_ignore_ascii_case("SELECT") {
        i += 1;
        let (columns, new_i) = parse_select_list(&tokens, i)?;
        i = new_i;

        if i >= tokens.len() || !tokens[i].eq_ignore_ascii_case("FROM") {
            return Err("Expected FROM".to_string());
        }

        i += 1;

        if i >= tokens.len() {
            return Err("Expected table name".to_string());
        }
        let table = tokens[i].clone();
        i += 1;

        let where_clause = if i < tokens.len() && tokens[i].eq_ignore_ascii_case("WHERE") {
            i += 1;
            let (expr, new_i) = parse_where_expr(&tokens, i)?;
            i = new_i;
            Some(expr)
        } else {
            None
        };

        if i < tokens.len() {
            return Err(format!("Extra tokens after query: {:?}", &tokens[i..]));
        }

        Ok(Statement::Select(Select {
            columns,
            table,
            where_clause,
        }))
    } else {
        Err("Unkonown command".to_string())
    }
}

fn parse_select_list(tokens: &[String], mut i: usize) -> Result<(Vec<ColumnRef>, usize), String> {
    let mut columns = Vec::new();

    loop {
        if i >= tokens.len() {
            return Err("Expected column or *".to_string());
        }

        if tokens[i] == "*" {
            columns.push(ColumnRef::Star);
            i += 1;
            break;
        } else {
            columns.push(ColumnRef::Name(tokens[i].clone()));
            i += 1;
        }

        if i < tokens.len() && tokens[i] == "," {
            i += 1;
        } else {
            break;
        }
    }

    Ok((columns, i))
}

fn parse_where_expr(tokens: &[String], mut i: usize) -> Result<(Expr, usize), String> {
    if i >= tokens.len() {
        return Err("Expected expression".to_string());
    }

    let left = Expr::Column(tokens[i].clone());
    i += 1;

    if i >= tokens.len() {
        return Err("Expected operator".to_string());
    }

    let op = match tokens[i].as_str() {
        "=" => BinaryOp::Eq,
        ">" => BinaryOp::Gt,
        "<" => BinaryOp::Lt,
        _ => return Err(format!("Unknown operator: {}", tokens[i])),
    };
    i += 1;

    if i >= tokens.len() {
        return Err("Expected value".to_string());
    }

    let right = if let Ok(n) = tokens[i].parse::<i64>() {
        Expr::Literal(Value::Integer(n))
    } else {
        Expr::Literal(Value::Text(tokens[i].clone()))
    };

    Ok((Expr::Binary(op, Box::new(left), Box::new(right)), i + 1))
}

pub fn tokenize(sql: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut buf = String::new();

    for c in sql.chars() {
        if c.is_whitespace() {
            if !buf.is_empty() {
                tokens.push(buf.clone());
                buf.clear();
            }
        } else if ",;()=<>".contains(c) {
            if !buf.is_empty() {
                tokens.push(buf.clone());
                buf.clear();
            }
            tokens.push(c.to_string());
        } else {
            buf.push(c);
        }
    }

    if !buf.is_empty() {
        tokens.push(buf);
    }

    tokens
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Select(Select),
    // Insert(Insert),
    // CreateTable(CreateTable),
}

#[derive(Debug, PartialEq)]
pub struct Select {
    pub columns: Vec<ColumnRef>,
    pub table: String,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, PartialEq)]
pub enum ColumnRef {
    Star,
    Name(String),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Column(String),
    Literal(Value),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Eq,
    Gt,
    Lt,
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Text(String),
    Null,
}
