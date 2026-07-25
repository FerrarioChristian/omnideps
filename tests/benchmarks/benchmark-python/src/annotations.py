def my_decorator(func):
    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)
    return wrapper

@my_decorator
def decorated_function():
    pass

class DataClassMeta:
    pass

@DataClassMeta
class MyDataClass:
    pass
