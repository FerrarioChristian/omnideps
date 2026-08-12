class Animal:
    def __init__(self, species: str) -> None:
        self.species = species
        
    def speak(self) -> None:
        pass

class Dog(Animal):
    def __init__(self, name: str) -> None:
        super().__init__("Dog")
        self.name = name
        
    def speak(self) -> None:
        print("Woof")

def test_inference() -> None:
    my_dog = Dog("Rex")
    my_dog.speak()
