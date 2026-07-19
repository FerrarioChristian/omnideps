from models import User, Admin
import utils

def main():
    admin = Admin("alice", 1990, "Moderator")
    
    admin.elevate_privileges()
    
    greeting = utils.greet_user(admin.username)
    age = utils.calculate_age(admin.birth_year, 2026)
    
    print(greeting)
    print(admin.get_info())
    print(f"Age: {age}")

if __name__ == "__main__":
    main()
