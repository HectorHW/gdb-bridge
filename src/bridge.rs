use memfile::MemFile;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::os::fd::AsRawFd;
use std::process::{Child, Command, Stdio};

use crate::parsing::{self, StopReason};

struct ProcessIO<I, O>
where
    I: BufRead,
    O: Write,
{
    stdin: O,
    stdout: I,
}

#[allow(dead_code)]
impl<I, O> ProcessIO<I, O>
where
    I: BufRead,
    O: Write,
{
    pub fn writeln(&mut self, data: &str) -> std::io::Result<()> {
        self.stdin.write_all(data.as_bytes())?;
        self.stdin.write_all("\n".as_bytes())?;
        self.stdin.flush()
    }

    pub fn write(&mut self, data: &str) -> std::io::Result<()> {
        self.stdin.write_all(data.as_bytes())?;
        self.stdin.flush()
    }

    pub fn read_line(&mut self) -> std::io::Result<String> {
        let mut buf = String::new();
        self.stdout.read_line(&mut buf)?;
        Ok(buf)
    }

    pub fn read_until(&mut self, prefix: &str) -> std::io::Result<String> {
        let mut buf = String::new();
        loop {
            let line = self.read_line()?;
            buf.push_str(&line);
            if line.starts_with(prefix) {
                break;
            }
        }
        Ok(buf)
    }

    pub fn read_all(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![];
        self.stdout.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

pub struct Gdb {
    child: Child,
    io: ProcessIO<BufReader<std::process::ChildStdout>, BufWriter<std::process::ChildStdin>>,
    exec_path: String,
}

#[derive(Debug)]
pub enum GDBSetupError {
    SpawnError(std::io::Error),
    IOError(std::io::Error),
    CommandError(std::io::Error),
}

#[derive(Debug)]
pub enum RunError {
    IOError(std::io::Error),
    ParseError(serde_json::Error),
}

impl From<std::io::Error> for RunError {
    fn from(value: std::io::Error) -> Self {
        RunError::IOError(value)
    }
}

impl From<serde_json::Error> for RunError {
    fn from(value: serde_json::Error) -> Self {
        RunError::ParseError(value)
    }
}

impl Gdb {
    pub fn new(exec_path: &str) -> Result<Self, GDBSetupError> {
        let mut child = Command::new("gdb")
            .arg("--interpreter=mi")
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(GDBSetupError::SpawnError)?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let io = ProcessIO {
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
        };

        let mut gdb = Gdb {
            child,
            io,
            exec_path: exec_path.to_string(),
        };

        gdb.io
            .read_until("(gdb)")
            .map_err(GDBSetupError::CommandError)?;

        gdb.setup_noninteractive()
            .map_err(GDBSetupError::CommandError)?;

        gdb.send_command(&format!("file {}", gdb.exec_path))
            .map_err(GDBSetupError::CommandError)?;

        Ok(gdb)
    }

    fn setup_noninteractive(&mut self) -> std::io::Result<()> {
        self.send_command("set pagination off")?;
        self.send_command("set debuginfod enabled off")?;
        self.send_command("set confirm off")?;
        Ok(())
    }

    pub fn send_command(&mut self, command: &str) -> std::io::Result<String> {
        self.io.writeln(command)?;
        self.io.read_line()?; // we skip line that is echo'd by mi interpreter
        self.io.read_until("(gdb)")
    }

    pub fn run(&mut self, input: &str) -> Result<(StopReason, Vec<u8>), RunError> {
        let mut stdin = MemFile::create_default("stdin")?;
        let mut stdout = MemFile::create_default("stdout")?;

        stdin.write_all(input.as_bytes())?;

        let this_id = std::process::id();

        let run_command = format!(
            "run </proc/{this_id}/fd/{} >/proc/{this_id}/fd/{}",
            stdin.as_raw_fd(),
            stdout.as_raw_fd()
        );

        self.send_command(&run_command)?;

        //now wait for process to either exit or crash

        let exit_result = self.io.read_until("*stopped")?;

        let last_line = exit_result.trim().split('\n').last().unwrap();
        let _ = self.io.read_until("(gdb)")?; // consume gdb from input

        if last_line.contains(r#"reason="signal-received""#) {
            self.send_command("kill")?;
        }

        let mut program_output = vec![];

        stdout.read_to_end(&mut program_output)?;

        Ok((parsing::parse_stop_message(last_line)?, program_output))
    }
}

impl Drop for Gdb {
    fn drop(&mut self) {
        //we should probably exit gdb, but what if we crash trying to exit? meh
        self.io.writeln("exit").unwrap(); //no (gdb) after this
        self.child.wait().unwrap();
    }
}
