from typing import TypeVar, Generic
from models import User
from advanced_models import SuperAdmin

# Generic class with TypeVar
T = TypeVar('T', bound=User)

class Container(Generic[T]):
    def __init__(self, item: T) -> None:
        self.item = item

# Class with generic collection fields
class UserStorage:
    def __init__(self, u: User, admin: SuperAdmin) -> None:
        self.container: Container[User] = Container(u)
        self.users: list[User] = [u]
        self.admin_map: dict[str, SuperAdmin] = {"root": admin}
        self.nested_users: list[list[User]] = [[u]]

# Bounded generic function with method invocation
def inspect_user_generic(u: T) -> str:
    return u.get_info()
