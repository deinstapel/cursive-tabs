
#[derive(Debug)]
pub struct IdNotFound;

impl std::error::Error for IdNotFound {}

impl std::fmt::Display for IdNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id not found")
    }
}
