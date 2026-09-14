pub trait Chain: Send + Sync + 'static {
    const CHAIN_ID: u64 = 0;
}

pub trait HasGrpc: Chain {}

pub struct Polygon;
pub struct Bnb;
pub struct Robinhood;

impl Chain for Polygon {
    const CHAIN_ID: u64 = 137;
}

impl Chain for Bnb {
    const CHAIN_ID: u64 = 56;
}

impl Chain for Robinhood {
    const CHAIN_ID: u64 = 4663;
}

impl HasGrpc for Polygon {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Base;
    impl Chain for Base {
        const CHAIN_ID: u64 = 8453;
    }

    struct Anything;
    impl Chain for Anything {}

    #[test]
    fn user_defined_chain() {
        assert_eq!(Base::CHAIN_ID, 8453);
        assert_eq!(Anything::CHAIN_ID, 0);
    }
}
