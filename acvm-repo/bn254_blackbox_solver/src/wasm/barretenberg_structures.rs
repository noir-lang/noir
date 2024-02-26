use acir::FieldElement;

#[derive(Debug, Default)]
pub(crate) struct Assignments(Vec<FieldElement>);

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

impl From<Vec<FieldElement>> for Assignments {
    fn from(w: Vec<FieldElement>) -> Assignments {
        Assignments(w)
    }
}
