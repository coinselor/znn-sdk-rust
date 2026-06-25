//! Client constants: default ports, retry count, and page sizes.

/// Number of connection retries before giving up.
pub const NUM_RETRIES: u32 = 10;

/// Maximum page size for paginated JSON-RPC responses.
pub const RPC_MAX_PAGE_SIZE: u32 = 1024;

/// Page size for memory-pool queries.
pub const MEMORY_POOL_PAGE_SIZE: u32 = 50;

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_expected_values() {
        assert_eq!(NUM_RETRIES, 10);
        assert_eq!(RPC_MAX_PAGE_SIZE, 1024);
        assert_eq!(MEMORY_POOL_PAGE_SIZE, 50);
    }
}
