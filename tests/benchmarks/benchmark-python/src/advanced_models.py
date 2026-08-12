from models import Admin
from typing import List

class SuperAdmin(Admin):
    def __init__(self, username: str, birth_year: int, role: str, permissions: List[str]) -> None:
        super().__init__(username, birth_year, role)
        self.permissions = permissions

    def grant_permission(self, perm: str) -> None:
        self.permissions.append(perm)

    @classmethod
    def create_root(cls, username: str) -> 'SuperAdmin':
        return cls(username, 1970, "Root", ["all"])

    @staticmethod
    def get_max_level() -> int:
        return 10
