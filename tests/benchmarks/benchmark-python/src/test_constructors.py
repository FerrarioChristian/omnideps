class Animal:
    def __init__(self, species: str):
        self.species = species
        
    def speak(self):
        pass

class Dog(Animal):
    def __init__(self, name: str):
        super().__init__("Dog")
        self.name = name
        
    def speak(self):
        print("Woof")

def test_inference():
    my_dog = Dog("Rex")
    my_dog.speak()

