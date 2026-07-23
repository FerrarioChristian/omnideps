from models import Admin
from typing import List

class SuperAdmin(Admin):
    def __init__(self, username, birth_year, role, permissions: List[str]):
        super().__init__(username, birth_year, role)
        self.permissions = permissions

    def grant_permission(self, perm: str):
        self.permissions.append(perm)

    @classmethod
    def create_root(cls, username: str) -> 'SuperAdmin':
        return cls(username, 1970, "Root", ["all"])

    @staticmethod
    def get_max_level() -> int:
        return 10
