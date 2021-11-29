use std::env;
use std::io::{self, BufRead as _, Write};

use anyhow::Context as _;
use console::Term;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

static PB_TICK_INTERVAL_MS: u64 = 50;
static PB_TEMPL_COUNT: &str =
    "{spinner:.green} {prefix} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} {per_sec} ETA {eta}";
static PB_TEMPL_BYTES: &str =
    "{spinner:.green} {prefix} [{elapsed_precise}] [{wide_bar:.cyan/blue}] \
     {bytes:>9}/{total_bytes:>9} {bytes_per_sec:>11} ETA {eta:>3}";
static PB_PROGRESS_CHARS: &str = "#>-";

#[derive(Debug)]
enum Inner {
    Term(Term),
    Buf {
        input: io::BufReader<io::Cursor<String>>,
        output: Vec<u8>,
    },
    Sink(io::Sink),
}

/// Config for console.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ConsoleConfig {
    /// If true, assumes yes and skips any confirmation.
    pub assume_yes: bool,
}

#[derive(Debug)]
pub struct Console {
    inner: Inner,
    conf: ConsoleConfig,
}

impl Console {
    pub fn term(conf: ConsoleConfig) -> Self {
        Self {
            inner: Inner::Term(Term::stderr()),
            conf,
        }
    }

    pub fn buf(conf: ConsoleConfig) -> Self {
        Self {
            inner: Inner::Buf {
                input: io::BufReader::new(io::Cursor::new(String::new())),
                output: Vec::new(),
            },
            conf,
        }
    }

    pub fn sink(conf: ConsoleConfig) -> Self {
        Self {
            inner: Inner::Sink(io::sink()),
            conf,
        }
    }

    #[cfg(test)]
    fn write_input(&mut self, s: &str) {
        if let Inner::Buf { ref mut input, .. } = self.inner {
            input.get_mut().get_mut().push_str(s)
        }
    }

    pub fn take_buf(self) -> Option<Vec<u8>> {
        match self.inner {
            Inner::Buf { output: buf, .. } => Some(buf),
            _ => None,
        }
    }

    pub fn take_output(self) -> crate::Result<String> {
        self.take_buf()
            .context("Could not take buf from console")
            .and_then(|buf| Ok(String::from_utf8(buf)?))
    }

    #[inline]
    fn as_mut_write(&mut self) -> &mut dyn Write {
        match self.inner {
            Inner::Term(ref mut w) => w,
            Inner::Buf {
                output: ref mut w, ..
            } => w,
            Inner::Sink(ref mut w) => w,
        }
    }

    pub fn warn(&mut self, message: &str) -> io::Result<()> {
        writeln!(self, "WARN: {}", message)
    }

    pub fn confirm(&mut self, message: &str, default: bool) -> io::Result<bool> {
        if self.conf.assume_yes {
            return Ok(true);
        }

        let prompt = format!("{} ({}) ", message, if default { "Y/n" } else { "y/N" });
        let input = self.prompt_and_read(&prompt, false)?;
        match input.to_lowercase().as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Ok(default),
        }
    }

    pub fn get_env_or_prompt_and_read(
        &mut self,
        env_name: &str,
        prompt: &str,
        is_password: bool,
    ) -> io::Result<String> {
        if let Ok(val) = env::var(env_name) {
            writeln!(
                self,
                "{}{:16} (read from env {})",
                prompt,
                if is_password { "********" } else { &val },
                env_name
            )?;
            return Ok(val);
        };
        self.prompt_and_read(prompt, is_password)
    }

    fn read_user(&mut self, is_password: bool) -> io::Result<String> {
        match self.inner {
            Inner::Term(ref term) => {
                if is_password {
                    term.read_secure_line()
                } else {
                    term.read_line()
                }
            }
            Inner::Buf { ref mut input, .. } => {
                let mut buf = String::new();
                input.read_line(&mut buf)?;
                Ok(buf)
            }
            Inner::Sink(_) => Ok(String::from("")),
        }
    }

    fn prompt(&mut self, prompt: &str) -> io::Result<()> {
        write!(self, "{}", prompt)?;
        self.flush()?;
        Ok(())
    }

    fn prompt_and_read(&mut self, prompt: &str, is_password: bool) -> io::Result<String> {
        self.prompt(prompt)?;
        self.read_user(is_password)
    }

    pub fn build_pb_count(&self, len: u64) -> ProgressBar {
        self.build_pb_with(len, PB_TEMPL_COUNT)
    }

    pub fn build_pb_bytes(&self, len: u64) -> ProgressBar {
        self.build_pb_with(len, PB_TEMPL_BYTES)
    }

    fn build_pb_with(&self, len: u64, template: &str) -> ProgressBar {
        let pb = ProgressBar::with_draw_target(len, self.to_pb_target());
        let style = Self::pb_style_common().template(template);
        pb.set_style(style);
        pb.enable_steady_tick(PB_TICK_INTERVAL_MS);
        pb
    }

    fn to_pb_target(&self) -> ProgressDrawTarget {
        match &self.inner {
            Inner::Term(term) => ProgressDrawTarget::to_term(term.clone(), None),
            _ => ProgressDrawTarget::hidden(),
        }
    }

    fn pb_style_common() -> ProgressStyle {
        ProgressStyle::default_bar().progress_chars(PB_PROGRESS_CHARS)
    }
}

impl Write for Console {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_mut_write().write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.as_mut_write().flush()
    }
}

macro_rules! def_color {
    ($name:ident, $name_upper:ident, $style:expr) => {
        ::lazy_static::lazy_static! {
            static ref $name_upper: ::console::Style = {
                use ::console::Style;
                $style
            };
        }

        pub fn $name<D>(val: D) -> ::console::StyledObject<D> {
            $name_upper.apply_to(val)
        }
    };
}

pub use color_defs::*;

#[cfg_attr(tarpaulin, ignore)]
mod color_defs {
    def_color!(sty_none, STY_NONE, Style::new());
    def_color!(sty_r, STY_R, Style::new().red());
    def_color!(sty_g, STY_G, Style::new().green());
    def_color!(sty_y, STY_Y, Style::new().yellow());
    def_color!(sty_dim, STY_DIM, Style::new().dim());
    def_color!(sty_r_under, STY_R_UNDER, Style::new().underlined().red());
    def_color!(sty_g_under, STY_G_UNDER, Style::new().underlined().green());
    def_color!(sty_y_under, STY_Y_UNDER, Style::new().underlined().yellow());
    def_color!(sty_r_rev, STY_R_REV, Style::new().bold().reverse().red());
    def_color!(sty_g_rev, STY_G_REV, Style::new().bold().reverse().green());
    def_color!(sty_y_rev, STY_Y_REV, Style::new().bold().reverse().yellow());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warn() -> anyhow::Result<()> {
        let conf = ConsoleConfig { assume_yes: true };
        let mut cnsl = Console::buf(conf);
        cnsl.warn("message")?;
        let output_str = cnsl.take_output()?;
        assert_eq!(output_str, "WARN: message\n");
        Ok(())
    }

    #[test]
    fn test_confirm() -> anyhow::Result<()> {
        let tests = &[
            (true, "", false, true),
            (false, "y", false, true),
            (false, "Y", false, true),
            (false, "yes", false, true),
            (false, "Yes", false, true),
            (false, "n", true, false),
            (false, "N", true, false),
            (false, "no", true, false),
            (false, "No", true, false),
            (false, "hoge", true, true),
            (false, "hoge", false, false),
            (false, "", true, true),
            (false, "", false, false),
        ];
        for (assume_yes, input, default, expected) in tests {
            let conf = ConsoleConfig {
                assume_yes: *assume_yes,
            };
            let mut cnsl = Console::buf(conf);
            cnsl.write_input(input);
            let actual = cnsl.confirm("message", *default).unwrap();
            assert_eq!(actual, *expected);
        }
        Ok(())
    }

    #[test]
    fn test_get_env_or_prompt_and_read() -> anyhow::Result<()> {
        let cnsl_term = Console::term(ConsoleConfig::default());
        let cnsl_buf_0 = Console::buf(ConsoleConfig::default());
        let mut cnsl_buf_1 = Console::buf(ConsoleConfig::default());
        cnsl_buf_1.write_input("test_input");
        let cnsl_sink_0 = Console::sink(ConsoleConfig::default());
        let cnsl_sink_1 = Console::sink(ConsoleConfig::default());
        let env_name_exists = if cfg!(windows) { "APPDATA" } else { "HOME" };
        let env_val: &str = &env::var(env_name_exists).unwrap();
        let tests = &mut [
            (cnsl_term, env_name_exists, env_val),
            (cnsl_buf_0, env_name_exists, env_val),
            (cnsl_buf_1, "ACICK_TEST_UNKNOWN_VAR", "test_input"),
            (cnsl_sink_0, env_name_exists, env_val),
            (cnsl_sink_1, "ACICK_TEST_UNKNOWN_VAR", ""),
        ];

        for (ref mut cnsl, env_name, expected) in tests {
            let actual = cnsl.get_env_or_prompt_and_read(env_name, "prompt >", true)?;
            assert_eq!(&actual, expected);
        }
        Ok(())
    }
}
