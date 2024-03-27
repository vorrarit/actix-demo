use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ActixDemoResponse<T> {
    pub data: T
}