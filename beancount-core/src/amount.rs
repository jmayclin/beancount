use rust_decimal::Decimal;
use std::cmp;
use std::convert::TryFrom;
use typed_builder::TypedBuilder;

use super::Currency;

/// A number of units of a certain commodity.
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder, Hash)]
pub struct Amount {
    /// The value of the amount.
    pub num: Decimal,

    /// The commodity of the amount.
    pub currency: Currency,
}

impl cmp::PartialOrd for Amount {
    fn partial_cmp(&self, other: &Amount) -> Option<cmp::Ordering> {
        if self.currency == other.currency {
            self.num.partial_cmp(&other.num)
        } else {
            None
        }
    }
}

/// An amount that may have missing units and/or commodity.
#[derive(Clone, Debug, Eq, PartialEq, Hash, TypedBuilder)]
pub struct IncompleteAmount {
    /// The (optional) value of the amount.
    #[builder(default)]
    pub num: Option<Decimal>,

    /// The (optional) commodity of the amount.
    #[builder(default)]
    pub currency: Option<Currency>,
}

impl cmp::PartialOrd for IncompleteAmount {
    fn partial_cmp(&self, other: &IncompleteAmount) -> Option<cmp::Ordering> {
        if self.currency == other.currency {
            self.num.partial_cmp(&other.num)
        } else {
            None
        }
    }
}

impl TryFrom<IncompleteAmount> for Amount {
    type Error = ();

    fn try_from(val: IncompleteAmount) -> Result<Self, Self::Error> {
        match val {
            IncompleteAmount {
                num: Some(num),
                currency: Some(currency),
            } => Ok(Amount { num, currency }),
            _ => Err(()),
        }
    }
}

impl From<Amount> for IncompleteAmount {
    fn from(val: Amount) -> Self {
        IncompleteAmount {
            num: Some(val.num),
            currency: Some(val.currency),
        }
    }
}
