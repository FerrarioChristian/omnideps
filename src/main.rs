// src/main.rs (versione minima per test)
use language_agnostic_analyzer::extractor::languages;

fn main() {
    let source = r#"
use tree_sitter::Parser

pub trait Entity {
    fn get_id(&self) -> String;
}

pub trait LivingBeing: Entity {
    fn breathe(&self);
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
    pub fn speak(&self) { println!("Meow"); }
    
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

    "#;

    let modules =
        language_agnostic_analyzer::extractor::generic_extract(languages::rust(), source).unwrap();

    println!("Trovati {} moduli", modules.len());
    for m in modules {
        println!(
            "  Modulo '{}' → {} structured types, {} free functions",
            m.name.join("::"),
            m.structured_types.len(),
            m.free_functions.len()
        );
    }
}
