// code taken from here:
// https://andres.svbtle.com/convert-subprocess-stdout-stream-into-non-blocking-iterator-in-rust

use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::io::BufReader;

pub struct SubProc {
    process: ::std::process::Child,
    tx: mpsc::Sender<Option<u8>>,
    rx: mpsc::Receiver<Option<u8>>,
}

impl SubProc {
    pub fn new() -> SubProc {
        let process =
                Command::new("sqlplus")
                        .args(&["/nolog"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn().unwrap();

        let (tx, rx) = mpsc::channel();
        SubProc {process, tx, rx, }
    }

    pub fn run(&mut self) {
        let tx = self.tx.clone();
        let stdout = self.process.stdout
                .take().unwrap();

        thread::spawn(move || {
            let reader = BufReader::new(stdout);

            for chunk in reader.bytes() {
                tx.send(Some(chunk.unwrap())).expect("unexpected");
            }
        });
    }

    pub fn push(&mut self, buf: &[u8]) {
        let stdin = self.process.stdin
                .as_mut().unwrap();

        stdin.write_all(buf).expect("unexpected");
    }

    pub fn packets(&mut self) -> SubProcessIntoIterator {
        SubProcessIntoIterator {
            subprocess: self,
        }
    }
}

pub struct SubProcessIntoIterator<'a> {
    subprocess: &'a mut SubProc,
}

impl <'a>Iterator for SubProcessIntoIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let data = self.subprocess.rx.try_recv();
        if data.is_ok() {
            data.unwrap()
        } else {
            None
        }
    }
}
