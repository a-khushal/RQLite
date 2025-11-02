pub fn plan(_ast: &Statement) -> Vec<Opcode> {
    vec![Opcode::Halt]
}

#[derive(Debug)]
pub enum Opcode {
    Halt,
}
