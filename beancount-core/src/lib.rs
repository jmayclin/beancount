
use typed_builder::TypedBuilder;

pub use account::Account;
pub use account_types::AccountType;
pub use amount::{Amount, IncompleteAmount};
pub use date::Date;
pub use directives::*;
pub use flags::Flag;
pub use position::CostSpec;
pub use posting::Posting;
pub use posting::PriceSpec;

pub mod account;
pub mod account_types;
pub mod amount;
mod date;
pub mod directives;
pub mod flags;
pub mod metadata;
pub mod position;
pub mod posting;
pub mod render;

/// Represents the complete ledger consisting of a number of directives.
// TODO: Derive Hash when possible
#[derive(Clone, Debug, PartialEq, Default, TypedBuilder)]
pub struct Ledger {
    pub directives: Vec<Directive>,
}

pub type Currency = String;
