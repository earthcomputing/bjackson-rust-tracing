/*---------------------------------------------------------------------------------------------
 *  Copyright © 2016-present Earth Computing Corporation. All rights reserved.
 *  Licensed under the MIT License. See LICENSE.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

// FIXME: should be a 'synch' operation for two events (i.e. happens before)

use std::collections::HashMap;
// use std::collections::hash_map::Entry;
use std::fmt;
use std::sync::{Mutex};
use std::thread;
use std::thread::ThreadId;

use lazy_static::lazy_static;
use serde_json;
use serde_json::{Value};
use time;

use failure::{Error};

pub fn parse(thread_id: ThreadId) -> u64 {
    let s = format!("{:?}", thread_id); // looks like : "ThreadId(8)"
    let r: Vec<&str> = s.split('(').collect();
    let parts: Vec<&str> = r[1].split(')').collect();
    parts[0].parse().expect(&format!("Problem parsing ThreadId {}", s))
}

pub fn timestamp() -> u64 {
    let timespec = time::get_time();
    let t = timespec.sec as f64 + (timespec.nsec as f64/1000./1000./1000.);
    (t*1000.0*1000.0) as u64
}

// --

lazy_static! {
    static ref EMITTERS: Mutex<HashMap<u64, EventGenerator>> = {
        let m = Mutex::new(HashMap::new());
        m
    };
}

pub fn debug(code: &CodeAttributes, body: &Value) -> Result<(String, String), Error> { under_lock(TraceType::Debug, code, body) }
pub fn trace(code: &CodeAttributes, body: &Value) -> Result<(String, String), Error> { under_lock(TraceType::Trace, code, body) }

// modifies 'clock' in place
fn under_lock(level: TraceType, code: &CodeAttributes, body: &Value) -> Result<(String, String), Error> {
    let tid = thread::current().id();
    let thread_id = parse(tid);
    let mut locked_map = EMITTERS.lock().unwrap();
    let v = locked_map.get_mut(&thread_id).unwrap();
    v.build_entry(level, code, body)
}

// updates 'clock', clones current setting
pub fn pregnant() -> EventGenerator {
    let tid = thread::current().id();
    let thread_id = parse(tid);
    let mut locked_map = EMITTERS.lock().unwrap();
    let v = locked_map.get_mut(&thread_id).unwrap();
    let mark = v.bump();
    let mut event_id = mark.event_id.clone();
    event_id.push(0);
    EventGenerator { thread_id: mark.thread_id, event_id: event_id, epoch: mark.epoch }
}

pub fn grandfather() {
    let mut e = EventGenerator::new();
    e.stash();
}

// --

#[derive(Debug, Copy, Clone, Serialize)]
enum TraceType { Trace, Debug, }

impl fmt::Display for TraceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Trace type {}", match self {
            &TraceType::Trace => "Trace",
            &TraceType::Debug => "Debug"
        })
    }
}

// --

#[derive(Debug, Clone, Serialize)]
pub struct CodeAttributes {
    pub module:   &'static str,
    pub function: &'static str,
    pub line_no:  u32,
    pub format:   &'static str
}

impl CodeAttributes {
    pub fn get_module(&self)   -> &'static str { self.module }
    pub fn get_function(&self) -> &'static str { self.function }
    pub fn get_line_no(&self)  -> u32          { self.line_no }
    pub fn get_format(&self)   -> &'static str { self.format }
}

// --

#[derive(Debug, Clone, Serialize)]
pub struct EventGenerator {
    thread_id: u64,
    event_id: Vec<u64>,
    epoch: u64
}

impl EventGenerator {
    pub fn new() -> EventGenerator {
        let tid = thread::current().id();
        let thread_id = parse(tid);
        let now = timestamp();
        let event_id = vec![0];

        EventGenerator { thread_id: thread_id, event_id: event_id, epoch: now, }
    }

    // monotonically increasing
    fn bump(&mut self) -> EventGenerator {
        let last = self.event_id.len() - 1;
        self.event_id[last] += 1;
        let now = timestamp();
        self.epoch = now;
        self.clone()
    }

    // creates a new thread 'clock' (epoch is parent's value)
    pub fn stash(&mut self) {
        let tid = thread::current().id();
        let thread_id = parse(tid);
        self.thread_id = thread_id;
        let mut locked_map = EMITTERS.lock().unwrap();
        locked_map.insert(thread_id, self.clone());
    }

    fn build_entry(&mut self, level: TraceType, code: &CodeAttributes, body: &Value) -> Result<(String, String), Error> {
        let mark = self.bump();
        let record = TraceRecord { header: &mark, level: &level, code: code, body: body };
        let doc = serde_json::to_string(&record)?;
        let key = format!("{:?}", &mark);
        Ok((key, doc))
    }
}

impl fmt::Display for EventGenerator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Thread id {}, Event id {:?} epoch {}", self.thread_id, self.event_id, self.epoch) }
}

// --

#[derive(Debug, Clone, Serialize)]
struct TraceRecord<'a> { header: &'a EventGenerator, level: &'a TraceType, code: &'a CodeAttributes, body: &'a Value }
