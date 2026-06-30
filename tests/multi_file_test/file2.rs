use crate::User;

pub struct Controller {
    pub user: User,
}

impl Controller {
    pub fn do_something(&self) {
        let n = self.user.get_name();
    }
}
