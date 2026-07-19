use tree_sitter::Parser

enum NodeType {
    Entity,
    LivingBeing,
    Animal,
    Mammal,
    Cat,
    Chargeable,
    Robot,
}

mod module {
    pub struct ModuleEntity {
        pub id: String,
    }
    
    impl super::Entity for ModuleEntity {
        fn get_id(&self) -> String { self.id.clone() }
    }
}

pub trait Entity {
    fn get_id(&self) -> String;
}

pub trait LivingBeing: Entity {
    fn breathe(&self);
}

fn useless_function() {
    struct LocalEntity {
        pub id: String,
    }
    
    impl Entity for LocalEntity {
        fn get_id(&self) -> String { self.id.clone() }
    }
    
    let entity = LocalEntity { id: "123".to_string() };
    entity.get_id();
}

pub struct Animal {
    pub name: String,
}

impl Entity for Animal {
    fn get_id(&self) -> String { self.name.clone() }
}

impl LivingBeing for Animal {
    fn breathe(&self) { println!("Breathing..."); }
}

pub struct Mammal {
    pub base: Animal,
}

pub struct Cat {
    pub base: Mammal,
}

impl Cat {
    pub fn speak(&self, to_whom: Animal) -> String { println!("Meow"); "meow".to_string() }
    
    pub struct Breed {
        pub species_type: String,
    }
}

pub trait Chargeable: Entity {
    fn charge(&self);
}

pub struct Robot {
    pub id: String,
}

impl Entity for Robot {
    fn get_id(&self) -> String { self.id.clone() }
}

impl Chargeable for Robot {
    fn charge(&self) { println!("Charging..."); }
}

// Moduli come Nesting
pub mod outer {
    pub mod inner {
        pub struct DeepInner;
        
        impl DeepInner {
            pub fn hello(&self) {
                // Local struct in function
                struct Local;
            }
        }
    }
}
