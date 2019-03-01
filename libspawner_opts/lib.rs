//! This library contains `CmdLineOptions` and `OptionValueParser` traits along with
//! some definitions used by `spawner_opts_macro` crate.
//!
//! # Examples
//! ```
//! use spawner_opts::*;
//!
//! #[derive(CmdLineOptions)]
//! #[optcont(
//!     delimeters = "=",
//!     usage = "tool [options]",
//! )]
//! struct Opts {
//!     #[flag(name = "-f", desc = "a flag")]
//!     flag: bool,
//!     
//!     #[opt(
//!         names("-v", "--v"),
//!         desc = "an option",
//!         value_desc = "<float>",
//!         parser = "FloatingLiteralParser"
//!     )]
//!     opt: f64,
//! }
//!
//! struct FloatingLiteralParser;
//!
//! impl OptionValueParser<f64> for FloatingLiteralParser {
//!     fn parse(opt: &mut f64, v: &str) -> Result<(), String> {
//!         match v.parse::<f64>() {
//!             Ok(x) => {
//!                 *opt = x;
//!                 Ok(())
//!             }
//!             Err(_) => Err(format!("Invalid value '{}'", v)),
//!         }
//!     }
//! }
//! ```

extern crate spawner_opts_derive;

pub mod parser;

pub use spawner_opts_derive::*;
use std::fmt;

pub struct OptionHelp {
    pub names: Vec<String>,
    pub desc: Option<String>,
    pub value_desc: Option<String>,
}

pub struct Help {
    pub overview: Option<String>,
    pub usage: Option<String>,
    pub delimeters: Option<String>,
    pub options: Vec<OptionHelp>,
}

pub trait CmdLineOptions: Sized {
    fn help() -> Help;
    fn parse<T, U>(&mut self, argv: T) -> Result<usize, String>
    where
        T: IntoIterator<Item = U>,
        U: AsRef<str>;
}

pub trait OptionValueParser<T> {
    fn parse(opt: &mut T, val: &str) -> Result<(), String>;
}

impl fmt::Display for Help {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref overview) = self.overview {
            write!(f, "Overview: {}\n\n", overview)?;
        }
        if let Some(ref usage) = self.usage {
            write!(f, "Usage: {}\n\n", usage)?;
        }
        if self.options.is_empty() {
            return Ok(());
        }

        let delim = match self.delimeters {
            Some(ref d) => d.chars().next().unwrap_or(' '),
            None => ' ',
        };
        f.write_str("Options:\n")?;
        for opt in self.options.iter() {
            write_opt(f, opt, delim)?;
        }
        Ok(())
    }
}

fn write_names(f: &mut fmt::Formatter, opt: &OptionHelp, delim: char) -> Result<usize, fmt::Error> {
    let mut names_len = 0;
    for (no, name) in opt.names.iter().enumerate() {
        if no > 0 {
            f.write_str(", ")?;
            names_len += 2;
        }
        f.write_str(name)?;
        names_len += name.len();
        if let Some(ref vd) = opt.value_desc {
            write!(f, "{}{}", delim, vd)?;
            names_len += 1 + vd.len();
        }
    }
    Ok(names_len)
}

fn write_opt(f: &mut fmt::Formatter, opt: &OptionHelp, delim: char) -> fmt::Result {
    let desc_offset = 30;
    let opt_offset = 2;
    let empty = &String::new();

    write_n(f, ' ', opt_offset)?;
    let written = opt_offset + write_names(f, opt, delim)?;

    for (no, line) in opt
        .desc
        .as_ref()
        .unwrap_or(empty)
        .split("\n")
        .filter(|line| !line.is_empty())
        .enumerate()
    {
        if no == 0 && written < desc_offset {
            write_n(f, ' ', desc_offset - written)?;
        } else {
            f.write_str("\n")?;
            write_n(f, ' ', desc_offset)?;
        }
        f.write_str(line)?;
    }
    f.write_str("\n")?;
    Ok(())
}

fn write_n(f: &mut fmt::Formatter, c: char, n: usize) -> fmt::Result {
    for _ in 0..n {
        write!(f, "{}", c)?;
    }
    Ok(())
}
