use std::borrow::Cow;

use rust_decimal::Decimal;
use typed_builder::TypedBuilder;

use super::amount::Amount;
use super::{Currency, Date};

#[derive(Clone, Debug, Eq, PartialEq, Hash, TypedBuilder)]
pub struct Cost {
    pub number: Decimal,
    pub currency: Currency,
    pub date: Date,
    pub label: Option<String>,
}

// TODO: Important Note. Amounts specified as either per-share or total prices or costs are always
// unsigned. It is an error to use a negative sign or a negative cost and Beancount will raise an
// error if you attempt to do so.

/// Represents a "cost", which typically belongs to a [Posting](struct.Posting.html).
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.mtqrwt24wnzs>
#[derive(Clone, Debug, Eq, PartialEq, Hash, TypedBuilder)]
pub struct CostSpec {
    #[builder(default)]
    pub number_per: Option<Decimal>,
    #[builder(default)]
    pub number_total: Option<Decimal>,
    /// The type of commodity for this cost.
    #[builder(default)]
    pub currency: Option<Currency>,
    /// The date of the at-cost.
    #[builder(default)]
    pub date: Option<Date>,
    /// The label of the cost.
    #[builder(default)]
    pub label: Option<String>,
    /// Flag to indicate that all lots should be merged and average cost to be used
    #[builder(default)]
    pub merge_cost: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, TypedBuilder)]
pub struct Position {
    pub units: Amount,
    pub cost: Option<Cost>,
}
