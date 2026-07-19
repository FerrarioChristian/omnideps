class User:
    def __init__(self, username, birth_year):
        self.username = username
        self.birth_year = birth_year

    def get_info(self):
        return f"User: {self.username}"

class Admin(User):
    def __init__(self, username, birth_year, role):
        super().__init__(username, birth_year)
        self.role = role

    def get_info(self):
        base_info = super().get_info()
        return f"{base_info}, Role: {self.role}"

    def elevate_privileges(self):
        self.role = "SuperAdmin"
