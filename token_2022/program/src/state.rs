// support the following extensions
// transfer fee - 0
// permanent delegate - 1
// interest bearing - 2
// non transferable - 4
// default account state - 8

pub enum Extensions {
    None = 0,
    TransferFee = 1,
    PermanentDelegate = 2,
    InterestBearing = 4,
    NonTransferable = 8,
    DefaultState = 16,
    TransferHook = 32,
}