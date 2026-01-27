use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)] // This is required for Soroban errors
pub enum Error {
    InvalidInput = 1,
    AlreadyExists = 2,
    NotFound = 3,
    UnsupportedToken = 4,
    InsufficientPayment = 5,
    NoUsagesRemaining = 6,
    InvalidUsageCount = 7,
    Unauthorized = 8,
}
