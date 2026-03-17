pub trait HasCardinality {
	const CARDINALITY: usize;
}

impl HasCardinality for u8 {
	const CARDINALITY: usize = 2usize.pow(Self::BITS);
}
