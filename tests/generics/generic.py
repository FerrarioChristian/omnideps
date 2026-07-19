class Entity:
    def __init__(self, id):
        self.id = id

class LivingBeing(Entity):
    def breathe(self):
        pass

class Animal(LivingBeing):
    def __init__(self, name):
        self.name = name

class Mammal(Animal):
    def give_birth(self):
        pass

class Cat(Mammal):
    def speak(self):
        return "Meow"
    
    # Esempio di Nested Class
    class Breed:
        def __init__(self, species_type):
            self.species_type = species_type

class Dog(Mammal):
    def speak(self):
        return "Woof"

class Robot(Entity):
    def charge(self):
        pass

# Esempio di Ereditarietà Multipla
class Cyborg(Mammal, Robot):
    def status(self):
        pass

# Esempio di Nesting profondo
class Outer:
    class Inner:
        class DeepInner:
            def hello(self):
                print("Hello from the deep")

# Classe definita dentro una funzione (Local Class)
def factory():
    class LocalProduct:
        pass
    return LocalProduct()
