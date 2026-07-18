use std::ops::Deref;

use crate::enums::EnumA;

#[derive(Clone)]
pub struct StructA {
  pub x: f64,
  pub y: f64
}

pub struct StructB {
  pub x: StructA,
  pub y: EnumA
}

pub struct StructC(pub StructA);

impl StructA {
  pub fn instance_method(&self) -> f64 {
    self.x * self.y
  }
  
  pub fn static_method() -> Self {
    StructA {x: 0.0, y: 0.0}
  }
}

impl StructB {
  pub fn instance_method(&self) -> f64 {
    self.x.instance_method()
  }
  pub fn instance_reference(&self) -> &StructA {
    &self.x
  }
}

impl Deref for StructC {
  type Target = StructA;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
