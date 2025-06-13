#[derive(Clone)]
pub struct NonNegativeOrderedFloat(pub f32);

impl PartialEq for NonNegativeOrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits().eq(&other.0.to_bits())
    }
}

impl Eq for NonNegativeOrderedFloat {}

impl PartialOrd for NonNegativeOrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.to_bits().partial_cmp(&other.0.to_bits())
    }
}

impl Ord for NonNegativeOrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.to_bits().cmp(&other.0.to_bits())
    }
}
