use std::{
    io::{self, Write},
};
fn main() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    println!("rqlite v1");

    loop {
        write!(handle, "db > ").unwrap();
        handle.flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        if buffer.trim().eq(".exit") {
            return;
        }

        exec_sql(&buffer);
    }
}

fn exec_sql(sql: &str) {
    let tokens = tokenize(sql);
    let ast = parse(tokens);
    println!("{:#?}", ast);
}

fn parse(tokens: Vec<String>) -> Result<Statement, String> {
    if tokens.is_empty() {
        return Err("Empty SQL query".to_string());
    }

    let mut i: usize = 0;

    if tokens[0].eq_ignore_ascii_case("select") {
        i += 1;
        let (columns, new_i) = parse_select_list(&tokens, i).unwrap();
        i = new_i;

        if i >= tokens.len() || !tokens[i].eq_ignore_ascii_case("from") {
            return Err("Expected FROM".to_string());
        }

        i += 1;
        if i >= tokens.len() {
            return Err("Expected table name".to_string());
        }
        let table = tokens[i].clone();
        i += 1;

        let where_clause = if i < tokens.len() && tokens[i].eq_ignore_ascii_case("where") {
            i += 1;
            let (expr, new_i) = parse_where_expr(&tokens, i).unwrap();
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
        return Err("Invalid SQL query".to_string());
    }
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
        _ => return Err(format!("Unknown opearator: {}", tokens[i])),
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
            columns.push(ColumnRef::Name(tokens[i].to_string()));
            i += 1
        }

        if i < tokens.len() && tokens[i] == "," {
            i += 1
        } else {
            break;
        }
    }

    Ok((columns, i))
}

fn tokenize(sql: &str) -> Vec<String> {
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
