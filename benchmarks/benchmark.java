package benchmarks;

interface Entity {
    String getId();
}

abstract class LivingBeing implements Entity {
    public abstract void breathe();
}

class Animal extends LivingBeing {
    protected String name;
    
    public Animal(String name) {
        this.name = name;
    }
    
    @Override
    public String getId() { return name; }
    
    @Override
    public void breathe() { System.out.println("Breathing..."); }
}

class Cat extends Animal {
    public Cat(String name) {
        super(name);
    }
    
    // Nested Static Class
    static class Breed {
        private String type;
        public Breed(String type) { this.type = type; }
    }
    
    // Inner Class
    class MeowBehavior {
        void speak() { System.out.println("Meow"); }
    }
}

// Interfaccia multipla (simula ereditarietà multipla)
interface Chargeable {
    void charge();
}

class Robot implements Entity, Chargeable {
    @Override
    public String getId() { return "Robot-1"; }
    @Override
    public void charge() { }
}

// Nesting profondo
class Outer {
    class Inner {
        class DeepInner {
            void hello() {
                // Local Class
                class Local { }
            }
        }
    }
}
