#[derive(Debug, Clone)]
pub struct Message {
    sender: usize,
    id: String,
}

impl Message {
    pub fn new(sender: usize, id: String) -> Self {
        Message { sender, id }
    }

    pub fn get_sender(&self) -> usize {
        self.sender
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
}
