use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TodoItem {
    pub description: String,
    pub done: Option<bool>
}
