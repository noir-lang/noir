use acvm::FieldElement;

#[derive(Debug, Default, Clone)]
pub(crate) struct Assignments(Vec<FieldElement>);

// This is a separate impl so the constructor can get the wasm_bindgen macro in the future
impl Assignments {
    #[allow(dead_code)]
    pub(crate) fn new() -> Assignments {
        Assignments::default()
    }
}

impl Assignments {
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        let witness_len = self.0.len() as u32;
        buffer.extend_from_slice(&witness_len.to_be_bytes());

        for assignment in self.0.iter() {
            buffer.extend_from_slice(&assignment.to_be_bytes());
        }

        buffer
    }
}

impl IntoIterator for Assignments {
    type Item = FieldElement;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<FieldElement>> for Assignments {
    fn from(w: Vec<FieldElement>) -> Assignments {
        Assignments(w)
    }
}
