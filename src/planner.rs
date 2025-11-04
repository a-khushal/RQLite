#![allow(dead_code)]

use crate::ast::{BinaryOp, ColumnRef, Expr, Select, Statement, Value};

#[derive(Debug, Clone)]
pub enum Opcode {
    OpenRead { root_page: u32 }, // Open B-tree cursor at root page
    Rewind,                      // Move cursor to first row
    Next,                        // Move to next row
    Column { index: usize },     // Load column value into register
    Integer(i64),                // Load integer constant
    Text(String),                // Load text constant
    Eq(usize, usize),            // Compare two registers (==)
    Gt(usize, usize),            // >
    Lt(usize, usize),            // <
    Goto(i32),                   // Relative jump
    ResultRow,                   // Emit current row
    MakeRecord,                  // Pack registers into row (for INSERT)
    Insert,                      // Insert row into B-tree
    Close,                       // Close cursor
    Halt,                        // Stop execution
}

pub fn plan(stmt: &Statement) -> Vec<Opcode> {
    match stmt {
        Statement::Select(select) => plan_select(select),
    }
}

fn plan_select(select: &Select) -> Vec<Opcode> {
    let mut code = Vec::new();
    let mut reg = 0;

    code.push(Opcode::OpenRead { root_page: 2 });

    let loop_start = code.len() as i32;
    code.push(Opcode::Rewind);

    let filter_jump_target = if let Some(expr) = &select.where_clause {
        let jump_offset = plan_where(&mut code, expr, &mut reg);
        code.len() as i32 + jump_offset
    } else {
        0
    };

    let mut output_regs = Vec::new();
    for col_ref in &select.columns {
        let col_idx = match col_ref {
            ColumnRef::Star => {
                let reg_id = reg;
                code.push(Opcode::Column { index: 0 });
                reg += 1;
                let reg_name = reg;
                code.push(Opcode::Column { index: 1 });
                reg += 1;
                let reg_age = reg;
                code.push(Opcode::Column { index: 2 });
                reg += 1;
                output_regs.extend([reg_id, reg_name, reg_age]);
                continue;
            }
            ColumnRef::Name(name) => match name.as_str() {
                "id" => 0,
                "name" => 1,
                "age" => 2,
                _ => 0,
            },
        };
        let reg_out = reg;
        code.push(Opcode::Column { index: col_idx });
        output_regs.push(reg_out);
        reg += 1;
    }

    code.push(Opcode::ResultRow);

    code.push(Opcode::Next);
    code.push(Opcode::Goto(loop_start - code.len() as i32 - 1));

    if filter_jump_target > 0 {
        let next_pc = code.len() as i32;
        code.push(Opcode::Next);
        code.push(Opcode::Goto(loop_start - code.len() as i32 - 1));

        if let Opcode::Goto(ref mut offset) = code[filter_jump_target as usize - 1] {
            *offset = next_pc - filter_jump_target;
        }
    }

    code.push(Opcode::Close);
    code.push(Opcode::Halt);

    code
}

fn plan_where(code: &mut Vec<Opcode>, expr: &Expr, reg: &mut usize) -> i32 {
    match expr {
        Expr::Binary(op, left, right) => {
            let col_idx = match **left {
                Expr::Column(ref name) => match name.as_str() {
                    "id" => 0,
                    "name" => 1,
                    "age" => 2,
                    _ => 0,
                },
                _ => 0,
            };
            let reg_left = *reg;
            code.push(Opcode::Column { index: col_idx });
            *reg += 1;

            let reg_right = *reg;
            match **right {
                Expr::Literal(Value::Integer(n)) => code.push(Opcode::Integer(n)),
                Expr::Literal(Value::Text(ref s)) => code.push(Opcode::Text(s.clone())),
                _ => code.push(Opcode::Integer(0)),
            }
            *reg += 1;

            match op {
                BinaryOp::Eq => code.push(Opcode::Eq(reg_left, reg_right)),
                BinaryOp::Gt => code.push(Opcode::Gt(reg_left, reg_right)),
                BinaryOp::Lt => code.push(Opcode::Lt(reg_left, reg_right)),
                _ => code.push(Opcode::Halt),
            }

            code.push(Opcode::Goto(0));
            3
        }
        _ => 0,
    }
}
