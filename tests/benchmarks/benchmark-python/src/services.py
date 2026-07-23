from advanced_models import SuperAdmin
from models import User

def register_admin(username: str) -> SuperAdmin:
    admin = SuperAdmin.create_root(username)
    admin.grant_permission("delete")
    return admin

def is_valid_user(user: User) -> bool:
    return len(user.username) > 3
