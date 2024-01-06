use serde::{
    Serialize,
    Deserialize,
};

#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Province {
    pub id: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct Holiday {
    pub provinces: Vec<Province>,
    pub observedDate: String
}
