use crate::structs::StructA;
use crate::structs::StructB;
use crate::structs::StructC;
use crate::enums::EnumA;
use crate::enums::EnumB;
use crate::traits::TraitA;
use crate::traits::TraitB;

static STATIC_SA: StructA = StructA {x: 1.0, y: 0.0};

pub type MyAlias = StructA;

pub fn function_with_local_variables() {
  let sa: StructA = StructA {x: 0.0, y: 0.0};
  let sb = StructB {x: sa.clone(), y: EnumA::FIRST};
  println!("{}", STATIC_SA.x);
}

pub fn function_with_alias(a: MyAlias) {
  println!("{}", a.x);
}

pub fn function_with_parameters(sa: &StructA) {
  println!("{}", sa.x);
}

pub fn function_with_return_types() -> EnumB {
  EnumB::FIRST(EnumA::SECOND)
}

pub fn function_with_type_casts() {
  function_with_return_types() as EnumB;
}

pub fn function_with_instance_methods() {
  let sa: StructA = StructA {x: 0.0, y: 0.0};
  println!("{}", sa.instance_method());
  let sb = StructB {x: sa.clone(), y: EnumA::FIRST};
  println!("{}", sb.instance_reference().instance_method());
}

pub fn function_with_static_methods() {
  let sa: StructA = StructA::static_method();
  println!("{}", sa.x);
}

pub fn function_with_deref_inheritance() {
  let sc: StructC = StructC(StructA::static_method());
  println!("{}", sc.instance_method());
  println!("{}", sc.x);
}

pub fn function_with_trait_methods<T: TraitA>(ta: T) {
  let sa: StructA = StructA::static_method();
  ta.trait_method(&sa);
}

pub fn function_with_inherited_trait_methods<T: TraitB>(tb: T) {
  let sa: StructA = StructA::static_method();
  tb.trait_method(&sa);
  let sb = StructB {x: sa.clone(), y: EnumA::FIRST};
  tb.new_trait_method(&sb);
}
