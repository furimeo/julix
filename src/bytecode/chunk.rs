use crate::bytecode::instruction::Instruction;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Instruction>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk { code: Vec::new() }
    }

    pub fn push(&mut self, instr: Instruction) {
        self.code.push(instr);
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.code.get(index)
    }
}

const MAGIC: &[u8] = b"JLXR";

pub fn serialize(chunk: &Chunk, path: &Path) -> std::io::Result<()> {
    let mut data = Vec::new();
    data.extend_from_slice(MAGIC);
    let count = chunk.code.len() as u32;
    data.extend_from_slice(&count.to_le_bytes());
    for instr in &chunk.code {
        instruction_to_bytes(instr, &mut data);
    }
    fs::write(path, data)
}

pub fn deserialize(path: &str) -> std::io::Result<Chunk> {
    let data = fs::read(path)?;
    if data.len() < 8 || &data[0..4] != MAGIC {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "not a valid .jlxr file",
        ));
    }
    let count = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let mut chunk = Chunk::new();
    let mut pos = 8;
    for _ in 0..count {
        let (instr, new_pos) = bytes_to_instruction(&data, pos);
        chunk.push(instr);
        pos = new_pos;
    }
    Ok(chunk)
}

fn instruction_to_bytes(instr: &Instruction, data: &mut Vec<u8>) {
    let (tag, payload) = match instr {
        Instruction::LoadInt(n) => (0u8, n.to_le_bytes().to_vec()),
        Instruction::LoadStr(s) => {
            let bytes = s.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            (1u8, payload)
        }
        Instruction::LoadBool(b) => (2u8, vec![if *b { 1 } else { 0 }]),
        Instruction::LoadConst(i) => (3u8, (*i as u32).to_le_bytes().to_vec()),
        Instruction::LoadVar(name) => {
            let bytes = name.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            (4u8, payload)
        }
        Instruction::StoreVar(name) => {
            let bytes = name.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            (5u8, payload)
        }
        Instruction::Add => (10u8, vec![]),
        Instruction::Sub => (11u8, vec![]),
        Instruction::Mul => (12u8, vec![]),
        Instruction::Div => (13u8, vec![]),
        Instruction::Mod => (14u8, vec![]),
        Instruction::Eq => (20u8, vec![]),
        Instruction::NotEq => (21u8, vec![]),
        Instruction::Lt => (22u8, vec![]),
        Instruction::Gt => (23u8, vec![]),
        Instruction::LtEq => (24u8, vec![]),
        Instruction::GtEq => (25u8, vec![]),
        Instruction::And => (30u8, vec![]),
        Instruction::Or => (31u8, vec![]),
        Instruction::Not => (32u8, vec![]),
        Instruction::Jump(t) => (40u8, (*t as u32).to_le_bytes().to_vec()),
        Instruction::JumpIfFalse(t) => (41u8, (*t as u32).to_le_bytes().to_vec()),
        Instruction::Print => (50u8, vec![]),
        Instruction::PrintLn => (51u8, vec![]),
        Instruction::Pop => (60u8, vec![]),
        Instruction::Halt => (99u8, vec![]),
    };
    data.push(tag);
    data.extend_from_slice(&payload);
}

fn bytes_to_instruction(data: &[u8], pos: usize) -> (Instruction, usize) {
    let tag = data[pos];
    let pos = pos + 1;
    match tag {
        0 => {
            let n = i64::from_le_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]);
            (Instruction::LoadInt(n), pos + 8)
        }
        1 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let s = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            (Instruction::LoadStr(s), pos + 4 + len)
        }
        2 => (Instruction::LoadBool(data[pos] == 1), pos + 1),
        3 => {
            let i = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            (Instruction::LoadConst(i), pos + 4)
        }
        4 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let s = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            (Instruction::LoadVar(s), pos + 4 + len)
        }
        5 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let s = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            (Instruction::StoreVar(s), pos + 4 + len)
        }
        10 => (Instruction::Add, pos),
        11 => (Instruction::Sub, pos),
        12 => (Instruction::Mul, pos),
        13 => (Instruction::Div, pos),
        14 => (Instruction::Mod, pos),
        20 => (Instruction::Eq, pos),
        21 => (Instruction::NotEq, pos),
        22 => (Instruction::Lt, pos),
        23 => (Instruction::Gt, pos),
        24 => (Instruction::LtEq, pos),
        25 => (Instruction::GtEq, pos),
        30 => (Instruction::And, pos),
        31 => (Instruction::Or, pos),
        32 => (Instruction::Not, pos),
        40 => {
            let t = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            (Instruction::Jump(t), pos + 4)
        }
        41 => {
            let t = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            (Instruction::JumpIfFalse(t), pos + 4)
        }
        50 => (Instruction::Print, pos),
        51 => (Instruction::PrintLn, pos),
        60 => (Instruction::Pop, pos),
        99 => (Instruction::Halt, pos),
        _ => {
            eprintln!("error: unknown bytecode tag {}", tag);
            std::process::exit(1);
        }
    }
}
