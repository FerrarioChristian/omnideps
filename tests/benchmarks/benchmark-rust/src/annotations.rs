#[macro_use]
extern crate serde_derive;

#[derive(Debug)]
pub struct MyStruct {
    #[serde(rename = "custom_field")]
    pub field: String,
}

#[inline(always)]
pub fn my_func() {}
