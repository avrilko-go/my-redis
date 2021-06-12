#[derive(Debug)]
pub struct Unknown {
    command_name: String,
}

impl Unknown {
    pub fn new(name: impl ToString) -> Unknown {
        Unknown {
            command_name: name.to_string()
        }
    }

    pub fn get_name(&self) -> &str {
        &self.command_name
    }
}