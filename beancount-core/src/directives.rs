use std::collections::HashSet;
use std::convert::TryFrom;

use rust_decimal::Decimal;
use typed_builder::TypedBuilder;

use super::account::Account;
use super::amount::Amount;
use super::flags::Flag;
use super::metadata::{Link, Meta, Tag};
use super::posting::Posting;
use super::{Currency, Date};

/// The set of booking methods for positions on accounts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Booking {
    /// Reject ambiguous matches with an error.
    Strict,

    /// Strict booking method, but disambiguate further with sizes. Reject ambiguous matches with
    /// an error but if a lot matches the size exactly, accept it the oldest.
    StrictWithSize,

    /// Disable matching and accept the creation of mixed inventories.
    None,

    /// Average cost booking: merge all matching lots before and after.
    Average,

    /// First-in first-out in the case of ambiguity.
    Fifo,

    /// Last-in first-out in the case of ambiguity.
    Lifo,
}

impl TryFrom<&str> for Booking {
    type Error = ();

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "STRICT" => Ok(Booking::Strict),
            "STRICT_WITH_SIZE" => Ok(Booking::StrictWithSize),
            "NONE" => Ok(Booking::None),
            "AVERAGE" => Ok(Booking::Average),
            "FIFO" => Ok(Booking::Fifo),
            "LIFO" => Ok(Booking::Lifo),
            _ => Err(()),
        }
    }
}

/// Enum of all directive types.
#[derive(Clone, Debug, PartialEq)]
pub enum Directive {
    Open(Open),
    Close(Close),
    Balance(Balance),
    Option(BcOption),
    Commodity(Commodity),
    Custom(Custom),
    Document(Document),
    Event(Event),
    Include(Include),
    Note(Note),
    Pad(Pad),
    Plugin(Plugin),
    Price(Price),
    Query(Query),
    Transaction(Transaction),
    Unsupported,
}

/// Represents a `balance` directive, which is a way for you to input your statement balance into
/// the flow of transactions.
///
/// It tells Beancount to verify that the number of units of a particular commodity in some account
/// should equal some expected value at some point in time.
///
/// The general format of the `balance` directive is:
///
/// ```text
/// YYYY-MM-DD balance Account  Amount
/// ```
///
/// Some example `balance` directives:
///
/// ```text
/// ; Check cash balances from wallet
/// 2014-08-09 balance Assets:Cash     562.00 USD
/// 2014-08-09 balance Assets:Cash     210.00 CAD
/// 2014-08-09 balance Assets:Cash      60.00 EUR
/// 2013-09-20 balance Assets:Investing:Funds     319.020 ~ 0.002 RGAGX
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.l0pvgeniwvq8>
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Balance {
    /// Date of the balance.
    pub date: Date,

    /// Account to check the balance of.
    pub account: Account,

    /// Amount to balance.
    pub amount: Amount,

    #[builder(default)]
    pub tolerance: Option<Decimal>,

    // diff_amount: Option<Amount>,
    /// Metadata attached to the balance directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a Beancount `option`, which are configuration points global to the file.
///
/// The general format of the `option` directive is:
///
/// ```text
/// option Name Value
/// ```
///
/// Example `option` directive:
///
/// ```text
/// option "title" "Ed’s Personal Ledger"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.e2iyrfrmstl>

#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct BcOption {
    /// Name of the option.
    pub name: String,

    /// Value of the option.
    pub val: String,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

impl BcOption {
    /// Determines if the current option specifies a root account name change.
    /// For example, the following line will rename the 'Assets' root account to 'Activa':
    /// ```beancount
    /// option "name_assets" "Activa"
    /// ```
    ///
    /// If this option is such a name change, this function will return the account type
    /// and the new account name. Otherwise, it will return `None`.
    pub fn root_name_change(&self) -> Option<(crate::AccountType, String)> {
        use crate::AccountType::*;
        match self.name.as_ref() {
            "name_assets" => Some((Assets, self.val.to_string())),
            "name_liabilities" => Some((Liabilities, self.val.to_string())),
            "name_equity" => Some((Equity, self.val.to_string())),
            "name_income" => Some((Income, self.val.to_string())),
            "name_expenses" => Some((Expenses, self.val.to_string())),
            _ => None,
        }
    }
}

/// Represents a `close` directive.  This directive signifies the closing of an account.
///
/// The general format of the `close` directive is:
///
/// ```text
/// YYYY-MM-DD close Account
/// ```
///
/// Example of a `close` directive:
///
/// ```text
/// ; Closing credit card after fraud was detected.
/// 2016-11-28 close Liabilities:CreditCard:CapitalOne
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.wf248e8stnac>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Close {
    /// Date the account was closed.
    pub date: Date,

    /// Account being closed.
    pub account: Account,

    /// Metadata attached to the close directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `commodity` directive.  This directive allows you to declare commodities,
/// although doing so is not required in order to use a commodity.
///
/// The purpose of this directive is to attach commodity-specific metadata fields on it, so that it
/// can be gathered by plugins later on.
///
/// The general format of the `commodity` directive is:
///
/// ```text
/// YYYY-MM-DD commodity Currency
/// ```
///
/// Some examples of `commodity` directives:
///
/// ```text
/// 1867-01-01 commodity CAD
///     name: "Canadian Dollar"
///     asset-class: "cash"
///
/// 2012-01-01 commodity HOOL
///     name: "Hooli Corporation Class C Shares"
///     asset-class: "stock"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.a3si01ejc035>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Commodity {
    /// Date the commodity was declared.
    pub date: Date,

    /// Commodity name.
    pub name: Currency,

    /// Metadata attached to the commodity directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `custom` directive, which is a generic directive provided to allow clients to
/// experiment with new features, e.g., budgeting.
///
/// The long-term plan for Beancount is to allow plugins and
/// external clients to define their own directive types, to be declared and validated by the
/// Beancount input language parser. In the meantime, a generic directive is provided for clients
/// to prototype new features, e.g., budgeting.
///
/// The grammar for this directive is flexible:
///
/// ```text
/// YYYY-MM-DD custom TypeName Value1 ...
/// ```
///
/// The first argument is a string and is intended to be unique to your directive. Think of this as
/// the type of your directive. Following it, you can put an arbitrary list of strings, dates,
/// booleans, amounts, and numbers.
///
/// Example custom directive:
///
/// ```text
/// 2014-07-09 custom "budget" "..." TRUE 45.30 USD
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.20klpeqb6ajy>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Custom {
    /// Date associated with the custom directive.
    pub date: Date,

    /// Custom directive name.
    pub name: String,

    /// Arbitrary number of custom directive arguments.
    pub args: Vec<String>,

    /// Metadata attached to the custom directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `document` directive.  A `document` directive can be used to attach an external
/// file to the journal of an account.
///
/// The general format of the `document` directive is:
///
/// ```text
/// YYYY-MM-DD document Account  PathToDocument
/// ```
///
/// Example of a `document` directive:
///
/// ```text
/// 2013-11-03 document Liabilities:CreditCard "/home/joe/stmts/apr-2014.pdf"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.w1ins9jk4mq3>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Document {
    /// Date the document was linked.
    pub date: Date,

    /// Account document is added to.
    pub account: Account,

    /// Filesystem path to the document.
    pub path: String,

    /// Tags associated with the document.
    #[builder(default)]
    pub tags: HashSet<Tag>,

    /// Links associated with the document.
    #[builder(default)]
    pub links: HashSet<Link>,

    /// Metadata attached to the document directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents an `event` directive.  `event` directives are used to track the value of some
/// variable of your choice over time - for example, your location.
///
/// The general format of the event `directive` is:
///
/// ```text
/// YYYY-MM-DD event Name Value
/// ```
///
/// An example of an `event` directive:
///
/// ```text
/// 2014-07-09 event "location" "Paris, France"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.tm5fxddlik5x>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Event {
    /// Date the event occurred.
    pub date: Date,

    /// Name of the event.
    pub name: String,

    /// New value of the event.
    pub description: String,

    /// Metadata attached to the event directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents an `include` directive.  The `include` directive, as it sounds, includes another
/// Beancount file into the current one, allowing you to arbitrarily split up your ledger files.
///
/// The general format is:
///
/// ```text
/// include Filename
/// ```
///
/// Example `include` directive:
///
/// ```text
/// include "path/to/include/file.beancount"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.86lelow4097r>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Include {
    /// Fully qualified filename, including any necessary path segments.
    pub filename: String,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `note` directive.  A `note` directive is simply used to attach a dated comment to
/// the journal of a particular account.
///
/// The general format of the `note` directive is:
///
/// ```text
/// YYYY-MM-DD note Account Description
/// ```
///
/// Example of a `note` directive:
///
/// ```text
/// 2013-11-03 note Liabilities:CreditCard "Called about fraudulent card."
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.c4cyaa6o6rqm>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Note {
    /// Date of the note.
    pub date: Date,

    /// Account being noted.
    pub account: Account,

    /// Note description.
    pub comment: String,

    /// Metadata attached to the note directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `open` directive.  This directive signifies the opening of an account.
///
/// Some examples of the `open` directive:
///
/// ```text
/// 1990-01-01 open Expenses:Restaurant
///
/// 2014-05-01 open Liabilities:CreditCard:CapitalOne     USD
///
/// 2015-02-01 open Assets:Cash:Pesos
///     description: "A shared account to contain our pocket of pesos"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.omdgvaikswd0>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Open {
    /// Date the account was opened.
    pub date: Date,

    /// Account being opened.
    pub account: Account,

    /// Commodities allowed for the opened account. An empty list means no restrictions on the
    /// allowed commodities.
    #[builder(default)]
    pub currencies: Vec<Currency>,

    /// Booking method. The default booking method for accounts is
    /// [`Booking::Strict`](enum.Booking.html).
    #[builder(default)]
    pub booking: Option<Booking>,

    /// Metadata attached to the open directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `pad` directive.  A `pad` directive automatically inserts a transaction that will
/// make the subsequent balance assertion succeed, if it is needed.
///
/// It inserts the difference needed to fulfill that balance assertion. (What “rubber space” is in
/// LaTeX, `pad` directives are to balances in Beancount.)
///
/// Some examples of the `pad` directive:
///
/// ```text
/// The general format of the Pad directive is:
///     YYYY-MM-DD pad Account AccountPad
/// ```
///
/// Example of a `pad` directive:
///
/// ```text
///     2014-06-01 pad Assets:BofA:Checking Equity:Opening-Balances
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.aw8ic3d8k8rq>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Pad {
    /// Date of the pad.
    pub date: Date,

    /// Account to pad into.
    pub pad_to_account: Account,

    /// Account to pad from.
    pub pad_from_account: Account,

    /// Metadata attached to the pad directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `plugin` directive.
///
/// In the Python version of Beancount, this would allow you to
/// specify an actual arbitrary Python program to programmatically transform directives as they are
/// parsed.  That is clearly not (easily) possible in this Rust implementation, but we will still
/// give you back any 'plugin' directives we found in the form of this struct.
///
/// The general format of the `plugin` directive is:
///
/// ```text
/// plugin ModuleName StringConfig
/// ```
///
/// Example of a `plugin` directive:
///
/// ```text
/// plugin "beancount.plugins.module_name" "configuration data"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.lxgs9ewvbt8k>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Plugin {
    /// Full module name of the plugin.
    pub module: String,

    /// Configuration data to be passed to the plugin.
    #[builder(default)]
    pub config: Option<String>,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `price` directive, which establishes the rate of exchange between one commodity and
/// another.
///
/// Beancount sometimes creates an in-memory data store of prices for each commodity.  One use of
/// of this database is to report unrealized gains on account holdings. `price` directives can be
/// used to provide data points for this database. A `price` directive establishes the rate of
/// exchange between one commodity (the base commodity) and another (the quote commodity).
///
/// The general format of the `price` directive is:
///
/// ```text
/// YYYY-MM-DD price Commodity Price
/// ```
///
/// This directive says: “The price of one unit of HOOL on July 9th, 2014 was 579.18 USD.”
///
/// ```text
/// 2014-07-09 price HOOL  579.18 USD
/// ```
///
/// `price` directives for currency exchange rates work the same way:
///
/// ```text
/// 2014-07-09 price USD  1.08 CAD
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.f78ym1dxtemh>
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Price {
    /// Date of the price specification.
    pub date: Date,

    /// The commodity being priced (a.k.a the base commodity).
    pub currency: Currency,

    /// Value the currency is being quoted at.
    pub amount: Amount,

    /// Metadata attached to the price directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `query` directive.  `query` directives allow you to insert a query in the usual
/// stream of transactions.
///
/// It can be convenient to be able to associate SQL queries in a Beancount
/// file to be able to run these as a report automatically, and `query` directives enable this.
///
/// The general format of the `query` directive is:
///
/// ```text
/// YYYY-MM-DD query Name SqlContents
/// ```
///
/// An example of a `query` directive:
///
/// ```text
/// 2014-07-09 query "france-balances" "
///     SELECT account, sum(position) WHERE ‘trip-france-2014’ in tags"
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.nw8fgvy4ub1w>
#[derive(Clone, Debug, Eq, PartialEq, TypedBuilder)]
pub struct Query {
    /// Date on which the query should be run.
    pub date: Date,

    /// Name of the query.
    pub name: String,

    /// Query contents.
    pub query_string: String,

    /// Metadata attached to the query directive.
    #[builder(default)]
    pub meta: Meta,

    /// Source string from the parsed input
    #[builder(default)]
    pub source: Option<String>,
}

/// Represents a `txn` (or `*` or `!`) directive.
///
/// A transaction can be signified by any of those three symbols, where `txn` and `*` both indicate
/// a completed transaction and `!` indicates an incomplete transaction.  The lines that follow the
/// first line of a transaction are for “Postings.”  You can read more about those in the
/// [Posting](struct.Posting.html) struct.
///
/// A transaction may have an optional “payee” and/or a “narration", where the payee is a string
/// that represents an external entity that is involved in the transaction.  A narration is a
/// description of the transaction that you write. It can be a comment about the context, the person
/// who accompanied you, some note about the product you bought... whatever you want it to be.
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.aki1rqfx1z8q>
///
/// Both transactions and postings can have metadata.  The general form for a transaction is:
///
/// ```text
/// YYYY-MM-DD [txn|Flag] [[Payee] Narration]
///     [Key: Value]
///     ...
///     [Flag] Account       Amount [{Cost}] [@ Price]
///         [Key: Value]
///         ...
/// ```
///
/// The two following transactions are equivalent:
///
/// ```text
/// 2014-05-05 txn "Cafe Mogador" "Lamb tagine with wine"
///     Liabilities:CreditCard:CapitalOne         -37.45 USD
///     Expenses:Restaurant
///
/// 2014-05-05 * "Cafe Mogador" "Lamb tagine with wine"
///     Liabilities:CreditCard:CapitalOne         -37.45 USD
///     Expenses:Restaurant
/// ```
///
/// And this is an incomplete transaction with a payee of "Seaworld", a narration of "Tickets",
/// and a single posting:
///
/// ```text
/// 2014-05-08 ! "Seaworld" "Tickets"
///     Liabilities:CreditCard:CapitalOne         -80.00 USD
/// ```
///
/// <https://docs.google.com/document/d/1wAMVrKIA2qtRGmoVDSUBJGmYZSygUaR0uOMW1GV3YE0/edit#heading=h.up4dj751q84w>
#[derive(Clone, Debug, PartialEq, TypedBuilder)]
pub struct Transaction {
    pub date: Date,

    /// Whether or not a transaction is considered complete.
    ///
    /// `*` or `txn`: Completed transaction, known amounts, “this looks correct.”
    /// `!`: Incomplete transaction, needs confirmation or revision, “this looks incorrect.”
    #[builder(default=Flag::Okay)]
    pub flag: Flag,

    /// Payee of this transaction.
    #[builder(default)]
    pub payee: Option<String>,

    /// Narration of this transaction.
    pub narration: String,

    /// Tags associated with the transaction.
    #[builder(default)]
    pub tags: HashSet<Tag>,

    /// Links associated with the transactions.
    #[builder(default)]
    pub links: HashSet<Link>,

    /// Postings belonging to this transaction.
    #[builder(default)]
    pub postings: Vec<Posting>,

    /// Metadata attached to the transaction.
    #[builder(default)]
    pub meta: Meta,

    #[builder(default)]
    pub source: Option<String>,
}
