pub struct User {
    pub name: String,
}

impl User {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}
