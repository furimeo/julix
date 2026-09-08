use crate::bytecode::instruction::Instruction;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub chunk: Chunk,
    pub params: Vec<u16>,
    pub slot_count: u16,
}

#[derive(Debug, Clone)]
pub struct FunctionTable {
    pub funcs: HashMap<String, FunctionDef>,
}

impl FunctionTable {
    pub fn new() -> Self {
        FunctionTable {
            funcs: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, func: FunctionDef) {
        self.funcs.insert(name, func);
    }

    pub fn get(&self, name: &str) -> Option<&FunctionDef> {
        self.funcs.get(name)
    }
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
        Instruction::LoadFloat(n) => (66u8, n.to_le_bytes().to_vec()),
        Instruction::LoadNull => (71u8, vec![]),
        Instruction::LoadStr(s) => {
            let bytes = s.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            (1u8, payload)
        }
        Instruction::LoadBool(b) => (2u8, vec![if *b { 1 } else { 0 }]),
        Instruction::LoadSlot(slot) => (4u8, slot.to_le_bytes().to_vec()),
        Instruction::StoreSlot(slot) => (5u8, slot.to_le_bytes().to_vec()),
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
        Instruction::Call(name, argc) => {
            let bytes = name.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            payload.extend_from_slice(&(*argc as u32).to_le_bytes());
            (52u8, payload)
        }
        Instruction::Return => (53u8, vec![]),
        Instruction::Construct(type_name, field_names) => {
            let bytes = type_name.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            payload.extend_from_slice(&(field_names.len() as u32).to_le_bytes());
            for name in field_names {
                let nb = name.as_bytes();
                payload.extend_from_slice(&(nb.len() as u32).to_le_bytes());
                payload.extend_from_slice(nb);
            }
            (54u8, payload)
        }
        Instruction::GetField(field) => {
            let bytes = field.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            (55u8, payload)
        }
        Instruction::SetField(field) => {
            let bytes = field.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            (56u8, payload)
        }
        Instruction::MethodCall(method, argc) => {
            let bytes = method.as_bytes();
            let mut payload = (bytes.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(bytes);
            payload.extend_from_slice(&(*argc as u32).to_le_bytes());
            (57u8, payload)
        }
        Instruction::Deinit => (61u8, vec![]),
        Instruction::Throw => (69u8, vec![]),
        Instruction::TryCatch(catch_ip, end_ip) => {
            let mut payload = Vec::new();
            payload.extend_from_slice(&(*catch_ip as u32).to_le_bytes());
            payload.extend_from_slice(&(*end_ip as u32).to_le_bytes());
            (70u8, payload)
        }
        Instruction::LoadBytes(b) => {
            let mut payload = (b.len() as u32).to_le_bytes().to_vec();
            payload.extend_from_slice(b);
            (62u8, payload)
        }
        Instruction::NewList(count) => (63u8, (*count as u32).to_le_bytes().to_vec()),
        Instruction::NewMap(count) => (67u8, (*count as u32).to_le_bytes().to_vec()),
        Instruction::IndexGet => (64u8, vec![]),
        Instruction::IndexSet => (68u8, vec![]),
        Instruction::Stringify => (65u8, vec![]),
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
        4 => {
            let slot = u16::from_le_bytes([data[pos], data[pos + 1]]);
            (Instruction::LoadSlot(slot), pos + 2)
        }
        5 => {
            let slot = u16::from_le_bytes([data[pos], data[pos + 1]]);
            (Instruction::StoreSlot(slot), pos + 2)
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
        52 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let name = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            let argc = u32::from_le_bytes([
                data[pos + 4 + len],
                data[pos + 5 + len],
                data[pos + 6 + len],
                data[pos + 7 + len],
            ]) as usize;
            (Instruction::Call(name, argc), pos + 4 + len + 4)
        }
        53 => (Instruction::Return, pos),
        54 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let type_name = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            let mut p = pos + 4 + len;
            let nfields =
                u32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]) as usize;
            p += 4;
            let mut field_names = Vec::new();
            for _ in 0..nfields {
                let nlen =
                    u32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]) as usize;
                p += 4;
                field_names.push(String::from_utf8_lossy(&data[p..p + nlen]).to_string());
                p += nlen;
            }
            (Instruction::Construct(type_name, field_names), p)
        }
        55 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let field = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            (Instruction::GetField(field), pos + 4 + len)
        }
        56 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let field = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            (Instruction::SetField(field), pos + 4 + len)
        }
        57 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let method = String::from_utf8_lossy(&data[pos + 4..pos + 4 + len]).to_string();
            let argc = u32::from_le_bytes([
                data[pos + 4 + len],
                data[pos + 5 + len],
                data[pos + 6 + len],
                data[pos + 7 + len],
            ]) as usize;
            (Instruction::MethodCall(method, argc), pos + 4 + len + 4)
        }
        61 => (Instruction::Deinit, pos),
        69 => (Instruction::Throw, pos),
        70 => {
            let catch_ip =
                u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
            let end_ip =
                u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
                    as usize;
            (Instruction::TryCatch(catch_ip, end_ip), pos + 8)
        }
        62 => {
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            (
                Instruction::LoadBytes(data[pos + 4..pos + 4 + len].to_vec()),
                pos + 4 + len,
            )
        }
        63 => {
            let count = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            (Instruction::NewList(count), pos + 4)
        }
        64 => (Instruction::IndexGet, pos),
        67 => {
            let count = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            (Instruction::NewMap(count), pos + 4)
        }
        68 => (Instruction::IndexSet, pos),
        65 => (Instruction::Stringify, pos),
        66 => {
            let n = f64::from_le_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]);
            (Instruction::LoadFloat(n), pos + 8)
        }
        71 => (Instruction::LoadNull, pos),
        60 => (Instruction::Pop, pos),
        99 => (Instruction::Halt, pos),
        _ => {
            eprintln!("error: unknown bytecode tag {}", tag);
            std::process::exit(1);
        }
    }
}
