pub struct AssetClass {
    pub id: i32,
    pub name: String,
    pub value: f32,
}

pub struct Class {
    pub id: i32,
    pub name: String,
    pub allocation: f32,
    pub value: f32,
}

pub struct Asset {
    pub name: String,
    pub enabled: bool,
    pub quantity: f32,
    pub value: f32,
}
