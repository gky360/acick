use std::env;
use std::io::{self, Write};

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
    Buf(Vec<u8>),
    Sink(io::Sink),
}

#[derive(Debug)]
pub struct Console {
    inner: Inner,
}

impl Console {
    pub fn term() -> Self {
        Self {
            inner: Inner::Term(Term::stderr()),
        }
    }

    pub fn buf() -> Self {
        Self {
            inner: Inner::Buf(Vec::new()),
        }
    }

    pub fn sink() -> Self {
        Self {
            inner: Inner::Sink(io::sink()),
        }
    }

    pub fn take_buf(self) -> Option<Vec<u8>> {
        match self.inner {
            Inner::Buf(buf) => Some(buf),
            _ => None,
        }
    }

    #[inline(always)]
    fn as_mut_write(&mut self) -> &mut dyn Write {
        match self.inner {
            Inner::Term(ref mut w) => w,
            Inner::Buf(ref mut w) => w,
            Inner::Sink(ref mut w) => w,
        }
    }

    pub fn warn(&mut self, message: &str) -> io::Result<()> {
        writeln!(self, "WARN: {}", message)
    }

    pub fn confirm(&mut self, message: &str, default: bool) -> io::Result<bool> {
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
        match &self.inner {
            Inner::Term(term) => {
                if is_password {
                    term.read_secure_line()
                } else {
                    term.read_line()
                }
            }
            _ => Ok(String::from("")),
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
            // Inner::Term(term) => ProgressDrawTarget::to_term(term.clone(), None),
            _ => ProgressDrawTarget::hidden(),
        }
    }

    fn pb_style_common() -> ProgressStyle {
        ProgressStyle::default_bar().progress_chars(PB_PROGRESS_CHARS)
    }
}

impl Write for Console {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.as_mut_write().write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        self.as_mut_write().flush()
    }
}
