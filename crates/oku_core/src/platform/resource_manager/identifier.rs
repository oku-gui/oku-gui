#[derive(Clone, Debug)]
pub enum ResourceIdentifier {
    Url(String),
    File(String),
}