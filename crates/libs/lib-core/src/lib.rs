#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    status: bool,
    size: usize,
    message: String,
}
