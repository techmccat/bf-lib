use crate::{check_brackets, Error, RuntimeError, bf::*};
use rand::{distributions::Alphanumeric, Rng};
use std::{env, fs, path::PathBuf};
use subprocess::{Exec, ExitStatus, Redirection};

pub fn run(
    program: &str,
    input: Option<String>,
    time: Option<std::time::Duration>,
    tmp_path: Option<PathBuf>,
) -> Result<String, Error> {
    let code = translate(program, input)?;
    let name = "bf".to_owned()
        + &rand::thread_rng()
            .sample_iter(Alphanumeric)
            .take(14)
            .collect::<String>();
    let basepath = if let Some(s) = tmp_path {
        s
    } else {
        env::current_dir().unwrap()
    };
    {
        let mut source = basepath.clone();
        source.push(name.clone() + ".rs");
        fs::write(source, code).unwrap();
    }
    let rustc = Exec::cmd("rustc")
        .arg("-Copt-level=3")
        .arg(name.clone() + ".rs")
        .stderr(Redirection::Pipe)
        .capture()
        .unwrap();
    if !rustc.success() {
        return Err(Error::Compile(rustc.stderr_str()));
    }
    let mut exe = basepath;
    if cfg!(windows) {
        exe.push(format!(r".\{}.exe", name))
    } else {
        exe.push(format!("./{}", name))
    };

    let result = if let Some(t) = time {
        let mut p = Exec::cmd(exe)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Pipe)
            .popen()
            .unwrap();
        match p.wait_timeout(t) {
            Ok(Some(ExitStatus::Exited(c))) => match c {
                0 => Ok(p
                    .communicate_start(None)
                    .read()
                    .unwrap()
                    .0
                    .unwrap_or(vec![])
                    .into_iter()
                    .map(|c| c as char)
                    .collect::<String>()
                    .trim()
                    .to_owned()),
                10 => Err(Error::Runtime(RuntimeError::InputTooShort)),
                _ => Err(Error::Runtime(RuntimeError::OutOfMemoryBounds)),
            },
            Ok(Some(_)) => Err(Error::Runtime(RuntimeError::Signal)),
            Ok(None) => {
                p.terminate().unwrap();
                Err(Error::Timeout)
            }
            Err(e) => Err(Error::Subprocess(e)),
        }
    } else {
        let p = Exec::cmd(exe)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Pipe)
            .capture()
            .unwrap();
        if let ExitStatus::Exited(c) = p.exit_status {
            match c {
                0 => Ok(p.stdout_str().trim().to_owned()),
                10 => Err(Error::Runtime(RuntimeError::InputTooShort)),
                _ => Err(Error::Runtime(RuntimeError::OutOfMemoryBounds)),
            }
        } else {
            Err(Error::Runtime(RuntimeError::Signal))
        }
    };
    cleanup(&name);
    result
}

pub fn translate(program: &str, input: Option<String>) -> Result<String, Error> {
    check_brackets(program)?;
    let i1 = program.to_inst()?;
    Ok(to_rust(i1, input))
}

fn cleanup(name: &str) {
    fs::remove_file(format!("{}.rs", name)).unwrap();
    fs::remove_file(if cfg!(windows) {
        format!(".\\{}.exe", name)
    } else {
        format!("./{}", name)
    })
    .unwrap();
}

fn to_rust(inst: Vec<Instruction>, input: Option<String>) -> String {
    const START: &str = "use std::num::Wrapping;
fn main() {
let mut _m = [Wrapping(0u8); 30000];
let (mut _p, mut _b) = (0usize, 0usize);
let mut _o = String::new();\n";
    const END: &str = "println!(\"{}\", _o);}";
    let mut code = if let Some(s) = input {
        format!("let _i = \"{}\";\n", s)
    } else {
        String::new()
    };
    for i in inst {
        match i {
            Instruction::Move(x) => {
                if x >= 0 {
                    code.push_str(&format!("_p += {};\n", x))
                } else {
                    code.push_str(&format!("_p -= {};\n", x.abs()))
                }
            }
            Instruction::Add(x) => {
                if x >= 0 {
                    code.push_str(&format!("_m[_p] += Wrapping({});\n", x))
                } else {
                    code.push_str(&format!("_m[_p] -= Wrapping({});\n", x.abs()))
                }
            }
            Instruction::Print => code.push_str("_o.push(_m[_p].0 as char)\n;"),
            Instruction::Read => code.push_str(
                "_m[_p] = { _b += 1; if let Some(c) = _i.as_bytes().get(_b-1) { Wrapping(*c)
} else { std::process::exit(10) }};\n",
            ),
            Instruction::LoopStart => code.push_str("while _m[_p].0 != 0 {\n"),
            Instruction::LoopEnd => code.push_str("}\n"),
        }
    }
    format!("{}{}{}", START, code, END)
}
