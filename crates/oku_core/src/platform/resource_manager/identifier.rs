#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum ResourceIdentifier {
    Url(String),
    File(String),
}