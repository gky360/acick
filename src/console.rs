use std::io::{self, Read, Write};
use std::{env, fmt};

use termion::input::TermRead as _;

pub struct Console<'a> {
    stdin: &'a mut dyn Read,
    stderr: &'a mut dyn Write,
}

impl<'a> Console<'a> {
    pub fn new(stdin: &'a mut dyn Read, stderr: &'a mut dyn Write) -> Self {
        Self { stdin, stderr }
    }

    pub fn get_env_or_prompt_and_read(
        &mut self,
        env_name: &str,
        prompt: &str,
        is_password: bool,
    ) -> io::Result<String> {
        if let Ok(val) = env::var(env_name) {
            writeln!(
                self.stderr,
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
        if is_password {
            self.stdin.read_passwd(&mut self.stderr)
        } else {
            self.read_line()
        }
        .and_then(|maybe_str| {
            maybe_str.ok_or_else(|| io::Error::new(io::ErrorKind::Interrupted, "Interrupted"))
        })
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
}

impl Read for Console<'_> {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl Write for Console<'_> {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stderr.write(buf)
    }

    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        self.stderr.flush()
    }
}

impl fmt::Debug for Console<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Console")
    }
}
