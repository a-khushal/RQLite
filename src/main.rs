mod ast;
mod parser;

use parser::{parse, tokenize};
use std::io::{self, Write};

fn main() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    println!("rqlite v0.1.0 - type .exit to quit");

    loop {
        write!(handle, "db > ").unwrap();
        handle.flush().unwrap();

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        let sql = buffer.trim();
        if sql == ".exit" {
            return;
        }

        exec_sql(sql);
    }
}

fn exec_sql(sql: &str) {
    let tokens = tokenize(sql);
    match parse(&tokens) {
        Ok(ast) => println!("{:#?}", ast),
        Err(e) => println!("Parse error: {}", e),
    }
}
