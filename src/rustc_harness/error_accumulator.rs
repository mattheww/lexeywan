//! Support for capturing errors emitted by rustc.

extern crate rustc_driver;
extern crate rustc_error_messages;
extern crate rustc_errors;
extern crate rustc_span;

use std::mem;
use std::sync::{Arc, Mutex};

use rustc_error_messages::DiagMessage;
use rustc_errors::translation::Translator;
use rustc_errors::{DiagCtxt, registry::Registry};
use rustc_span::source_map::SourceMap;

#[derive(Clone)]
/// Storage for a list of error messages emitted by rustc.
///
/// This wraps an `Arc`; all clones modify the same list.
pub struct ErrorAccumulator {
    contents: Arc<Mutex<Vec<String>>>,
}

impl ErrorAccumulator {
    /// Returns a new empty error accumulator.
    pub fn new() -> Self {
        Self {
            contents: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Returns the accumulated error messages.
    pub fn extract(&self) -> Vec<String> {
        mem::take(&mut self.contents.lock().unwrap())
    }

    /// Says whether any error messages have been accumulated.
    pub fn has_any_errors(&self) -> bool {
        !self.contents.lock().unwrap().is_empty()
    }

    /// Returns an implementator of `rustc_errors::emitter::Emitter` which stores emitted errors
    /// into this accumulator.
    pub fn into_error_emitter(self) -> Box<impl rustc_errors::emitter::Emitter> {
        Box::new(ErrorEmitter::new(self))
    }

    /// Returns a `rustc_errors::DiagCtxt` which stores emitted errors into this accumulator.
    ///
    /// The `DiagCtxt` ignores non-error diagnostics.
    pub fn into_diag_ctxt(self) -> DiagCtxt {
        DiagCtxt::new(self.into_error_emitter())
    }

    /// Adds a non-rustc error message to the accumulator.
    pub fn push(&self, msg: String) {
        self.contents.lock().unwrap().push(msg);
    }
}

struct ErrorEmitter {
    translator: Translator,
    accumulator: ErrorAccumulator,
}

impl ErrorEmitter {
    fn new(error_list: ErrorAccumulator) -> Self {
        ErrorEmitter {
            translator: rustc_driver::default_translator(),
            accumulator: error_list,
        }
    }
}

impl rustc_errors::emitter::Emitter for ErrorEmitter {
    fn source_map(&self) -> Option<&SourceMap> {
        None
    }

    fn emit_diagnostic(&mut self, diag: rustc_errors::DiagInner, _: &Registry) {
        if !diag.is_error() {
            return;
        }
        let mut messages = self.accumulator.contents.lock().unwrap();
        if let Some(code) = diag.code {
            messages.push(format!("code: {code}"));
        } else if diag.messages.is_empty() {
            // I don't think this happens, but in case it does we store a
            // message so the caller knows to report failure.
            messages.push("error with no message".into());
        }
        for (msg, _style) in &diag.messages {
            let s = match msg {
                DiagMessage::Str(msg) => msg.to_string(),
                DiagMessage::Translated(msg) => msg.to_string(),
                DiagMessage::FluentIdentifier(fluent_id, _) => fluent_id.to_string(),
            };
            messages.push(s);
        }
    }

    fn translator(&self) -> &rustc_errors::translation::Translator {
        &self.translator
    }
}
