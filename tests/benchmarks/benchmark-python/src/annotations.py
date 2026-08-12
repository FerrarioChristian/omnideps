def my_decorator(func):
    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)
    return wrapper

@my_decorator
def decorated_function() -> None:
    pass

class DataClassMeta:
    pass

@DataClassMeta
class MyDataClass:
    pass
