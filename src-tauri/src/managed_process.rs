use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use sshtunnel_core::ssh_launch::{CommandSpec, LaunchPlan};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct ManagedProcess {
    inner: ManagedProcessInner,
    logs: Arc<Mutex<Vec<String>>>,
    needs_connection_signal: bool,
}

enum ManagedProcessInner {
    Native(NativeProcess),
    Prompted(PromptedProcess),
}

struct NativeProcess {
    child: Child,
    reader_thread: Option<JoinHandle<()>>,
}

struct PromptedProcess {
    child: Box<dyn portable_pty::Child + Send>,
    reader_thread: Option<JoinHandle<()>>,
}

impl ManagedProcess {
    pub fn spawn(plan: LaunchPlan) -> Result<Self, String> {
        match plan {
            LaunchPlan::Native(command) => Self::spawn_native(command),
            LaunchPlan::PromptedPassword {
                command,
                password,
                prompt,
            } => Self::spawn_prompted(command, &password, &prompt),
        }
    }

    pub fn pid(&self) -> Option<u32> {
        match &self.inner {
            ManagedProcessInner::Native(child) => Some(child.child.id()),
            ManagedProcessInner::Prompted(child) => child.child.process_id(),
        }
    }

    pub fn try_wait(&mut self) -> Result<Option<String>, String> {
        match &mut self.inner {
            ManagedProcessInner::Native(child) => {
                let status = child.child.try_wait().map_err(|error| error.to_string())?;
                if status.is_some() {
                    join_reader(&mut child.reader_thread);
                }
                Ok(status.map(|item| format!("{item}")))
            }
            ManagedProcessInner::Prompted(child) => {
                let status = child.child.try_wait().map_err(|error| error.to_string())?;
                if status.is_some() {
                    join_reader(&mut child.reader_thread);
                }
                Ok(status.map(|item| format!("{item:?}")))
            }
        }
    }

    pub fn kill(&mut self) -> Result<(), String> {
        match &mut self.inner {
            ManagedProcessInner::Native(child) => {
                child.child.kill().map_err(|error| error.to_string())?;
                let _ = child.child.wait();
                join_reader(&mut child.reader_thread);
            }
            ManagedProcessInner::Prompted(child) => {
                child.child.kill().map_err(|error| error.to_string())?;
                let _ = child.child.wait();
                join_reader(&mut child.reader_thread);
            }
        }

        Ok(())
    }

    pub fn take_logs(&self) -> Vec<String> {
        let mut logs = self.logs.lock().expect("managed-process logs poisoned");
        std::mem::take(&mut *logs)
    }

    pub fn needs_connection_signal(&self) -> bool {
        self.needs_connection_signal
    }

    fn spawn_native(command: CommandSpec) -> Result<Self, String> {
        let mut child = Command::new(&command.program);
        child.args(command.args);
        child.stdin(Stdio::null());
        child.stdout(Stdio::null());
        child.stderr(Stdio::piped());
        apply_hidden_process_creation(&mut child);

        let mut child = child.spawn().map_err(|error| error.to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "failed to capture stderr pipe from native process".to_string())?;
        let logs = Arc::new(Mutex::new(Vec::new()));
        let reader_thread = spawn_native_stderr_reader(stderr, Arc::clone(&logs));

        Ok(Self {
            inner: ManagedProcessInner::Native(NativeProcess {
                child,
                reader_thread: Some(reader_thread),
            }),
            logs,
            needs_connection_signal: false,
        })
    }

    fn spawn_prompted(command: CommandSpec, password: &str, prompt: &str) -> Result<Self, String> {
        let pty = native_pty_system();
        let pair = pty
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| error.to_string())?;

        let mut builder = CommandBuilder::new(command.program);
        for arg in command.args {
            builder.arg(arg);
        }

        let child = pair
            .slave
            .spawn_command(builder)
            .map_err(|error| error.to_string())?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| error.to_string())?;
        let mut writer = pair
            .master
            .take_writer()
            .map_err(|error| error.to_string())?;

        let logs = Arc::new(Mutex::new(Vec::new()));
        let password = password.to_string();
        let prompt = prompt.to_ascii_lowercase();
        let log_sink = Arc::clone(&logs);

        let reader_thread = thread::spawn(move || {
            let mut buf = [0_u8; 1024];
            let mut transcript = String::new();
            let mut sent_password = false;

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        if !transcript.trim().is_empty() {
                            push_log(&log_sink, transcript.trim());
                        }
                        break;
                    }
                    Ok(read) => {
                        let chunk = String::from_utf8_lossy(&buf[..read]).to_string();
                        transcript.push_str(&chunk);
                        for line in chunk.lines() {
                            if !line.trim().is_empty() {
                                push_log(&log_sink, line.trim());
                            }
                        }

                        if !sent_password
                            && transcript.to_ascii_lowercase().contains(&prompt)
                            && writer.write_all(password.as_bytes()).is_ok()
                            && writer.write_all(b"\n").is_ok()
                        {
                            let _ = writer.flush();
                            sent_password = true;
                            push_log(&log_sink, "password sent to interactive ssh session");
                        }
                    }
                    Err(error) => {
                        push_log(&log_sink, &format!("pty reader error: {error}"));
                        break;
                    }
                }
            }
        });

        Ok(Self {
            inner: ManagedProcessInner::Prompted(PromptedProcess {
                child,
                reader_thread: Some(reader_thread),
            }),
            logs,
            needs_connection_signal: true,
        })
    }
}

#[cfg_attr(not(any(target_os = "windows", test)), allow(dead_code))]
fn windows_process_creation_flags() -> u32 {
    #[cfg(target_os = "windows")]
    {
        CREATE_NO_WINDOW
    }

    #[cfg(not(target_os = "windows"))]
    {
        0
    }
}

fn apply_hidden_process_creation(command: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(windows_process_creation_flags());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = command;
    }
}

fn spawn_native_stderr_reader(
    stderr: impl Read + Send + 'static,
    logs: Arc<Mutex<Vec<String>>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        push_log(&logs, trimmed);
                    }
                }
                Err(error) => {
                    push_log(&logs, &format!("stderr reader error: {error}"));
                    break;
                }
            }
        }
    })
}

fn join_reader(reader_thread: &mut Option<JoinHandle<()>>) {
    if let Some(handle) = reader_thread.take() {
        let _ = handle.join();
    }
}

fn push_log(logs: &Arc<Mutex<Vec<String>>>, line: &str) {
    let mut guard = logs.lock().expect("managed-process logs poisoned");
    guard.push(line.to_string());
    if guard.len() > 24 {
        let trim = guard.len() - 24;
        guard.drain(0..trim);
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use sshtunnel_core::ssh_launch::{CommandSpec, LaunchPlan};

    use super::{windows_process_creation_flags, ManagedProcess};

    #[test]
    fn native_process_captures_stderr_lines() {
        let mut process = ManagedProcess::spawn(LaunchPlan::Native(stderr_echo_command()))
            .expect("spawn native stderr writer");
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut lines = Vec::new();

        while Instant::now() < deadline {
            lines.extend(process.take_logs());
            if lines.iter().any(|line| line.contains("stderr-first"))
                && lines.iter().any(|line| line.contains("stderr-second"))
            {
                break;
            }

            if process.try_wait().expect("query child status").is_some() && !lines.is_empty() {
                break;
            }

            std::thread::sleep(Duration::from_millis(25));
        }

        let _ = process.kill();

        assert!(
            lines.iter().any(|line| line.contains("stderr-first")),
            "expected stderr-first in logs, got {lines:?}"
        );
        assert!(
            lines.iter().any(|line| line.contains("stderr-second")),
            "expected stderr-second in logs, got {lines:?}"
        );
    }

    #[test]
    fn windows_process_creation_flags_match_platform_behavior() {
        #[cfg(target_os = "windows")]
        assert_eq!(windows_process_creation_flags(), 0x0800_0000);

        #[cfg(not(target_os = "windows"))]
        assert_eq!(windows_process_creation_flags(), 0);
    }

    #[cfg(target_os = "windows")]
    fn stderr_echo_command() -> CommandSpec {
        CommandSpec {
            program: "cmd".into(),
            args: vec![
                "/C".into(),
                "(echo stderr-first 1>&2) && (echo stderr-second 1>&2)".into(),
            ],
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn stderr_echo_command() -> CommandSpec {
        CommandSpec {
            program: "sh".into(),
            args: vec![
                "-c".into(),
                "printf 'stderr-first\\nstderr-second\\n' >&2".into(),
            ],
        }
    }
}
