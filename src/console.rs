use std::io::{self, Write};
use std::{env, fmt};

use rpassword::read_password;

pub struct Console<'a> {
    stderr: &'a mut dyn Write,
}

impl<'a> Console<'a> {
    pub fn new(stderr: &'a mut dyn Write) -> Self {
        Self { stderr }
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
            read_password()
        } else {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf)?;

            if buf.ends_with('\n') {
                // Remove the \n from the line if present
                buf.pop();
                // Remove the \r from the line if present
                if buf.ends_with('\r') {
                    buf.pop();
                }
            }
            Ok(buf)
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
