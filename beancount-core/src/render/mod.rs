use crate::*;
use metadata::MetaValue;
use std::collections::HashMap;
use std::{io, io::Write};
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct BasicRenderer {}

impl BasicRenderer {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn render<W: Write>(w: &mut W, ledger: &Ledger) -> Result<(), BasicRendererError> {
    BasicRenderer::default().render(ledger, w)
}

#[derive(Error, Debug)]
pub enum BasicRendererError {
    #[error("an io error occurred")]
    Io(#[from] io::Error),
    #[error("could not render unsupported directive")]
    Unsupported,
}

pub trait Renderer<T, W: Write> {
    type Error;
    fn render(&self, renderable: T, write: &mut W) -> Result<(), Self::Error>;
}

impl<'a, W: Write> Renderer<&'a Ledger, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, ledger: &'a Ledger, write: &mut W) -> Result<(), Self::Error> {
        for directive in &ledger.directives {
            self.render(directive, write)?;
            writeln!(write)?;
        }
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Document, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, document: &'a Document, write: &mut W) -> Result<(), Self::Error> {
        // TODO: Tags? Links?
        write!(write, "{} document ", document.date)?;
        self.render(&document.account, write)?;
        writeln!(write, " \"{}\"", document.path)?;
        render_key_value(self, write, &document.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Directive, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, directive: &'a Directive, write: &mut W) -> Result<(), Self::Error> {
        use Directive::*;
        match directive {
            Open(open) => self.render(open, write),
            Close(close) => self.render(close, write),
            Balance(balance) => self.render(balance, write),
            Option(bc_option) => self.render(bc_option, write),
            Commodity(commodity) => self.render(commodity, write),
            Custom(custom) => self.render(custom, write),
            Document(document) => self.render(document, write),
            Event(event) => self.render(event, write),
            Include(include) => self.render(include, write),
            Note(note) => self.render(note, write),
            Pad(pad) => self.render(pad, write),
            Plugin(plugin) => self.render(plugin, write),
            Price(price) => self.render(price, write),
            Query(query) => self.render(query, write),
            Transaction(transaction) => self.render(transaction, write),
            Unsupported => Err(BasicRendererError::Unsupported),
        }
    }
}

fn render_key_value<W: Write>(
    renderer: &BasicRenderer,
    w: &mut W,
    kv: &HashMap<String, MetaValue>,
) -> Result<(), BasicRendererError> {
    for (key, value) in kv {
        write!(w, "\t{}: ", key)?;
        renderer.render(value, w)?;
        writeln!(w)?;
    }
    Ok(())
}

impl<'a, W: Write> Renderer<&'a MetaValue, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, mv: &'a MetaValue, w: &mut W) -> Result<(), Self::Error> {
        match mv {
            MetaValue::Account(account) => self.render(account, w)?,
            MetaValue::Amount(amount) => self.render(amount, w)?,
            MetaValue::Bool(b) => write!(w, "{}", if *b { "true" } else { "false" })?,
            MetaValue::Currency(curr) => write!(w, "{}", curr)?,
            MetaValue::Date(date) => write!(w, "{}", date)?,
            MetaValue::Number(num) => write!(w, "{}", num)?,
            MetaValue::Tag(t) => write!(w, "{}", t)?,
            MetaValue::Text(t) => write!(w, "{}", t)?,
        }
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Open, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, open: &'a Open, write: &mut W) -> Result<(), Self::Error> {
        write!(write, "{} open ", open.date)?;
        self.render(&open.account, write)?;
        for currency in open.currencies.iter() {
            write!(write, " {}", currency)?;
        }
        match open.booking {
            Some(Booking::Strict) => write!(write, r#" "STRICT""#)?,
            Some(Booking::StrictWithSize) => write!(write, r#" "STRICT_WITH_SIZE""#)?,
            Some(Booking::None) => write!(write, r#" "NONE""#)?,
            Some(Booking::Average) => write!(write, r#" "AVERAGE""#)?,
            Some(Booking::Fifo) => write!(write, r#" "FIFO""#)?,
            Some(Booking::Lifo) => write!(write, r#" "LIFO""#)?,
            None => {}
        };
        writeln!(write)?;
        render_key_value(self, write, &open.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Close, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, close: &'a Close, write: &mut W) -> Result<(), Self::Error> {
        write!(write, "{} close ", close.date)?;
        self.render(&close.account, write)?;
        writeln!(write)?;
        render_key_value(self, write, &close.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Account, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, account: &'a Account, write: &mut W) -> Result<(), Self::Error> {
        write!(
            write,
            "{}:{}",
            account.ty.default_name(),
            account.parts.join(":")
        )?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Balance, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, balance: &'a Balance, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} balance ", balance.date)?;
        self.render(&balance.account, w)?;
        write!(w, "\t")?;
        write!(w, "{}", balance.amount.num)?;
        if let Some(tol) = balance.tolerance {
            write!(w, " ~ {}", tol)?;
        }
        writeln!(w, "{}", balance.amount.currency)?;
        render_key_value(self, w, &balance.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Amount, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, amount: &'a Amount, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} {}", amount.num, amount.currency)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a BcOption, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, option: &'a BcOption, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "option \"{}\" \"{}\"", option.name, option.val)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Commodity, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, commodity: &'a Commodity, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "{} commodity {}", commodity.date, commodity.name)?;
        render_key_value(self, w, &commodity.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Custom, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, custom: &'a Custom, w: &mut W) -> Result<(), Self::Error> {
        write!(
            w,
            "{} custom \"{}\" {}",
            custom.date,
            custom.name,
            custom.args.join(" ")
        )?;
        writeln!(w)?;
        render_key_value(self, w, &custom.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Event, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, event: &'a Event, w: &mut W) -> Result<(), Self::Error> {
        writeln!(
            w,
            "{} event \"{}\" \"{}\"",
            event.date, event.name, event.description
        )?;
        render_key_value(self, w, &event.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Include, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, include: &'a Include, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "include \"{}\'", include.filename)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Note, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, note: &'a Note, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} note ", note.date)?;
        self.render(&note.account, w)?;
        writeln!(w, " {}", note.comment)?;
        render_key_value(self, w, &note.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Pad, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, pad: &'a Pad, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} pad ", pad.date)?;
        self.render(&pad.pad_to_account, w)?;
        write!(w, " ")?;
        self.render(&pad.pad_from_account, w)?;
        writeln!(w)?;
        render_key_value(self, w, &pad.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Plugin, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, plugin: &'a Plugin, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "plugin \"{}\"", plugin.module)?;
        if let Some(config) = &plugin.config {
            write!(w, " \"{}\"", config)?;
        }
        writeln!(w)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Price, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, price: &'a Price, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} price {} ", price.date, price.currency)?;
        self.render(&price.amount, w)?;
        writeln!(w)?;
        render_key_value(self, w, &price.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Query, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, query: &'a Query, w: &mut W) -> Result<(), Self::Error> {
        writeln!(
            w,
            "{} query \"{}\" \"{}\"",
            query.date, query.name, query.query_string
        )?;
        render_key_value(self, w, &query.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Transaction, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, transaction: &'a Transaction, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} {}", transaction.date, transaction.flag)?;
        if let Some(payee) = &transaction.payee {
            write!(w, " \"{}\"", payee)?;
        }
        write!(w, " \"{}\"", &transaction.narration)?;
        for tag in &transaction.tags {
            write!(w, " {}", tag)?;
        }
        for link in &transaction.links {
            write!(w, " {}", link)?;
        }
        writeln!(w)?;
        for posting in &transaction.postings {
            self.render(posting, w)?;
        }
        render_key_value(self, w, &transaction.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Posting, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, posting: &'a Posting, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "\t")?;
        if let Some(flag) = &posting.flag {
            write!(w, "{} ", flag)?;
        }
        self.render(&posting.account, w)?;
        write!(w, "\t")?;
        self.render(&posting.units, w)?;
        if let Some(cost) = &posting.cost {
            write!(w, " ")?;
            self.render(cost, w)?;
        }
        if let Some(price) = &posting.price {
            write!(w, " ")?;
            self.render(price, w)?;
        }
        writeln!(w)?;
        render_key_value(self, w, &posting.meta)
    }
}

impl<'a, W: Write> Renderer<&'a CostSpec, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, cost: &'a CostSpec, w: &mut W) -> Result<(), Self::Error> {
        let double_brackets = cost.number_total.is_some();
        if double_brackets {
            write!(w, "{{{{")?;
        } else {
            write!(w, "{{")?;
        }
        let mut first = true;

        if let (Some(cost), Some(currency)) =
            (&cost.number_total.or(cost.number_per), &cost.currency)
        {
            write!(w, "{} {}", cost, currency)?;
            first = false;
        }

        if let Some(date) = &cost.date {
            if !first {
                write!(w, ", ")?;
            }
            write!(w, "{}", date)?;
            first = false;
        }

        if let Some(label) = &cost.label {
            if !first {
                write!(w, ", ")?;
            }
            write!(w, "{}", label)?;
        }

        if double_brackets {
            write!(w, "}}}}")?;
        } else {
            write!(w, "}}")?;
        }
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a PriceSpec, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, price: &'a PriceSpec, w: &mut W) -> Result<(), Self::Error> {
        let amount = match price {
            PriceSpec::PerUnit(amount) => {
                write!(w, "@ ")?;
                amount
            }
            PriceSpec::Total(amount) => {
                write!(w, "@@ ")?;
                amount
            }
        };
        self.render(amount, w)
    }
}

impl<'a, W: Write> Renderer<&'a IncompleteAmount, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(
        &self,
        incomplete_amount: &'a IncompleteAmount,
        w: &mut W,
    ) -> Result<(), Self::Error> {
        match (&incomplete_amount.num, &incomplete_amount.currency) {
            (Some(num), Some(currency)) => write!(w, "{} {}", num, currency),
            (None, Some(currency)) => write!(w, "{}", currency),
            (Some(num), None) => write!(w, "{}", num),
            _ => write!(w, ""),
        }?;
        Ok(())
    }
}
