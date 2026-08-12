from models import User, Admin
import utils
import services

def main() -> None:
    admin = Admin("alice", 1990, "Moderator")
    
    admin.elevate_privileges()
    
    greeting = utils.greet_user(admin.username)
    age = utils.calculate_age(admin.birth_year, 2026)
    
    print(greeting)
    print(admin.get_info())
    print(f"Age: {age}")

    super_admin = services.register_admin("bob")
    is_valid = services.is_valid_user(super_admin)
    print(f"Is valid: {is_valid}")

if __name__ == "__main__":
    main()
