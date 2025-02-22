use typed_builder::TypedBuilder;

use super::account::Account;
use super::amount::IncompleteAmount;
use super::flags::Flag;
use super::metadata::Meta;
use super::position::CostSpec;

/// Represents a transaction posting.  Postings represent a single amount being deposited to or
/// withdrawn from an account.
///
/// Postings can have optionally have either a cost or a price.  A posting with a price might look
/// like this, where the price is the amount and commodity following the `@`:
///
/// ```text
/// 2012-11-03 * "Transfer to account in Canada"
///     Assets:MyBank:Checking            -400.00 USD @ 1.09 CAD
///     Assets:FR:SocGen:Checking          436.01 CAD
/// ```
///
/// A posting with a cost is the same with the exception that it utilizes `@@`.
///
/// ```text
/// 2012-11-03 * "Transfer to account in Canada"
///     Assets:MyBank:Checking            -400.00 USD @@ 436.01 CAD
///     Assets:FR:SocGen:Checking          436.01 CAD
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.mtqrwt24wnzs>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Posting {
    /// Account being posted to.
    pub account: Account,

    /// The amount being posted.
    /// TODO: Remove this
    pub units: IncompleteAmount,

    /// The cost of this posting.
    #[builder(default)]
    pub cost: Option<CostSpec>,

    /// The price of this posting.
    #[builder(default)]
    pub price: Option<PriceSpec>,

    #[builder(default)]
    pub flag: Option<Flag>,

    #[builder(default)]
    pub meta: Meta,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PriceSpec {
    PerUnit(IncompleteAmount),
    Total(IncompleteAmount),
}
