use crate::structs::StructA;

pub enum EnumA {
  FIRST,
  SECOND,
  THIRD
}

pub enum EnumB {
  FIRST(EnumA),
  SECOND{b: StructA}
}
