use crate::lixvm::value::Value;

#[derive(Clone)]
pub struct Environment {
    slots: Vec<Value>,
}

impl Environment {
    pub fn new(slot_count: usize) -> Self {
        Environment {
            slots: vec![Value::Null; slot_count],
        }
    }

    pub fn get_slot(&self, index: u16) -> &Value {
        &self.slots[index as usize]
    }

    pub fn set_slot(&mut self, index: u16, value: Value) {
        if (index as usize) >= self.slots.len() {
            self.slots.resize(index as usize + 1, Value::Null);
        }
        self.slots[index as usize] = value;
    }

    pub fn objects(&self) -> Vec<Value> {
        self.slots
            .iter()
            .filter_map(|v| match v {
                Value::Object(_, _) => Some(v.clone()),
                _ => None,
            })
            .collect()
    }
}
