#[derive(Debug, Clone)]
pub struct Message {
    id: String,
}

impl Message {
    pub fn new(id: String) -> Self {
        Message { id }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
