class User:
    def __init__(self, username: str, birth_year: int) -> None:
        self.username = username
        self.birth_year = birth_year

    def get_info(self) -> str:
        return f"User: {self.username}"

class Admin(User):
    def __init__(self, username: str, birth_year: int, role: str) -> None:
        super().__init__(username, birth_year)
        self.role = role

    def get_info(self) -> str:
        base_info = super().get_info()
        return f"{base_info}, Role: {self.role}"

    def elevate_privileges(self) -> None:
        self.role = "SuperAdmin"
