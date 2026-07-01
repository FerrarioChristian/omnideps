use crate::enums::EnumA;
use crate::structs::StructA;
use crate::enums::EnumB;
use crate::structs::StructB;

pub trait TraitA {
  fn trait_method(&self, sa: &StructA) -> EnumA;
}

pub trait TraitB: TraitA {
  fn new_trait_method(&self, sb: &StructB) -> EnumB;
}

