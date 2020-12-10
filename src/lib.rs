//! # bf-lib
//!
//! `bf-lib` is small library to run brainfuck programs non-interactively
//!
//! The entry point is the [`Exec`] struct, stole the idea from [`subprocess`]
//!
//! [`subprocess`]: https://crates.io/crates/subprocess

use std::{error, fmt, time, path::PathBuf};

/// Possible errors encountered while running the program.
#[derive(Debug)]
pub enum Error {
    Compile(String),
    Runtime(RuntimeError),
    Subprocess(subprocess::PopenError),
    Syntax(usize),
    Timeout,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pre = "Error, I didn't quite get that.\n";
        match self {
            Error::Compile(s) => write!(f, "{}rustc error: {}", pre, s),
            Error::Runtime(e) => write!(f, "{}Runtime error: {}", pre, e),
            Error::Subprocess(e) => write!(f, "{}rustc error: {}", pre, e),
            Error::Syntax(p) => write!(f, "{}Unmatched bracket at {}.", pre, p),
            Error::Timeout => write!(f, "{}Executable timed out.", pre),
        }
    }
}

/// Possible runtime errors encountered while running the program.
#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    OutOfMemoryBounds,
    InputTooShort,
    Signal,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::OutOfMemoryBounds => write!(f, "access memory out of bounds"),
            RuntimeError::InputTooShort => write!(f, "input was not long enough"),
            RuntimeError::Signal => write!(f, "executable was probably killed by a signal"),
        }
    }
}

/// Interface for running brainfuck code.
///
/// The [`prog`] method returns an instance with the default options (no timeout, input or
/// temporary file path)
///
/// [`input`], [`timeout`] and [`tmpdir`] are used to change the default values, the program can
/// then be run by calling [`run`], [`transpile`] or [`interpret`].
///
/// [`prog`]: struct.Exec.html#method.prog
/// [`input`]: struct.Exec.html#method.input
/// [`timeout`]: struct.Exec.html#method.timeout
/// [`tmpdir`]: struct.Exec.html#method.tmpdir
/// [`run`]: struct.Exec.html#method.run
/// [`transpile`]: struct.Exec.html#method.transpile
/// [`interpret`]: struct.Exec.html#method.interpret
/// ```
/// # use bf_lib::Exec;
/// let prog = "++++++++++[>++++++++++>+++++++++++<<-]>++.>+..";
/// let output = Exec::prog(prog).run().unwrap();
///
/// assert_eq!(String::from("foo"), output);
/// ```
pub struct Exec {
    program: String,
    input: Option<String>,
    time: Option<time::Duration>,
    tmp_path: Option<PathBuf>,
}

impl Exec {
    /// Contructs a new `Exec`, configured to run `prog`.
    /// By default it will be run without input, timeout or temporary file path (defaults to cwd).
    pub fn prog(prog: &str) -> Exec {
        Exec {
            program: String::from(prog),
            input: None,
            time: None,
            tmp_path: None,
        }
    }

    /// Sets the input for the program.
    pub fn input(self, input: Option<String>) -> Exec {
        Exec {
            input,
            ..self
        }
    }

    /// Sets the timeout for the program.
    pub fn timeout(self, time: Option<time::Duration>) -> Exec {
        Exec {
            time,
            ..self
        }
    }

    /// Sets the temporary file path for the transpiler.
    pub fn tmpdir(self, tmp_path: Option<PathBuf>) -> Exec {
        Exec {
            tmp_path,
            ..self
        }
    }
    
    /// Wrapper for the [`transpile`] and [`interpret`] methods:
    /// uses the faster transpiler when rustc is detected, falls back to interpreting the code.
    ///
    /// [`transpile`]: struct.Exec.html#method.interpret
    /// [`interpret`]: struct.Exec.html#method.transpile
    pub fn run(self) -> Result<String, Error> {
        bf::run(&self.program, self.input, self.time, self.tmp_path)
    }
    
    /// Runs the program with the interpreter, returning the output or an [`Error`].
    pub fn interpret(self) -> Result<String, Error> {
        bf::interpreter::run(&self.program, self.input, self.time)
    }
    
    /// Runs the program with the transpiler, returning the output or an [`Error`].
    ///
    /// Needs read and write permission in the chosen temporary file folder.
    pub fn transpile(self) -> Result<String, Error> {
        bf::transpiler::run(&self.program, self.input, self.time, self.tmp_path)
    }

    /// Translated the program to rust code
    pub fn translate(&self) -> Result<String, Error> {
        bf::transpiler::translate(&self.program, self.input.clone())
    }
}

/// Looks for unmatched brackets
///
/// ```
/// let ok = "+[+]";
/// let err = "[[]";
/// bf_lib::check_brackets(ok).unwrap();
/// bf_lib::check_brackets(err).unwrap_err();
/// ```
pub fn check_brackets(prog: &str) -> Result<(), Error> {
    let mut open: Vec<usize> = Vec::new();
    for (i, b) in prog.as_bytes().iter().enumerate() {
        match b {
            b'[' => open.push(i),
            b']' => {
                if let None = open.pop() {
                    return Err(Error::Syntax(i));
                };
            }
            _ => (),
        }
    }
    if open.len() != 0 {
        Err(Error::Syntax(open.pop().unwrap()))
    } else {
        Ok(())
    }
}

/// Checks if the program will try to read user input.
///
/// ```
/// let reads = ",[>+>+<<-]>.>.";
/// let does_not_read = "foo. bar.";
///
/// assert_eq!(true, bf_lib::wants_input(reads));
/// assert_eq!(false, bf_lib::wants_input(does_not_read));
/// ```
pub fn wants_input(program: &str) -> bool {
    program.contains(",")
}

mod bf;

#[cfg(test)]
mod tests;
