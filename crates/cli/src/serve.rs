// Copyright (C) 2024 Red Hat
// SPDX-License-Identifier: Apache-2.0

//! This module provides a bare minimal http server for reading report with xdg-open

use anyhow::Result;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

fn get_listener(port: u16) -> Option<(u16, TcpListener)> {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => Some((port, l)),
        _ if port < 10_000 => None,
        _ => get_listener(port + 1),
    }
}

pub fn serve(name: &str, index: &str, report: &logjuicer_report::Report) -> Result<()> {
    let mut report_bytes: Vec<u8> = Vec::new();
    report.save_writer(&mut report_bytes)?;
    let index_line = format!("GET /{name} ");
    match get_listener(8000) {
        None => Err(anyhow::anyhow!("Couldn't find available port!")),
        Some((port, listener)) => {
            let url = format!("http://127.0.0.1:{port}/{name}");
            match std::process::Command::new("xdg-open").arg(&url).spawn() {
                Ok(_) => {}
                Err(e) => {
                    println!("Failed to start xdg-open {url} : {e}");
                }
            }
            for stream in listener.incoming() {
                let mut stream = stream?;

                let buf_reader = BufReader::new(&mut stream);
                let request_line = buf_reader.lines().next().unwrap()?;
                let status_line = "HTTP/1.1 200 OK";
                println!("{}", request_line);
                let content = if request_line.starts_with(&index_line) {
                    index.as_bytes()
                } else {
                    &report_bytes
                };
                let length = content.len();
                let headers = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n");

                stream.write_all(headers.as_bytes())?;
                stream.write_all(content)?;
                if !request_line.starts_with(&index_line) {
                    break;
                };
            }
            Ok(())
        }
    }
}
