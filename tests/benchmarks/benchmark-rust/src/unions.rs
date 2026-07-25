pub union MyUnion {
    pub f1: u32,
    pub f2: f32,
}

pub fn handle_union(u: MyUnion) -> u32 {
    unsafe { u.f1 }
}
