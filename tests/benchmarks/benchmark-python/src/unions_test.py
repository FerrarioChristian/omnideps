from typing import Union

class Dog:
    pass

class Cat:
    pass

def handle_animal(animal: Dog | Cat) -> None:
    pass

def process_legacy(animal: Union[Dog, Cat]) -> None:
    pass
