//! Support for capturing errors emitted by rustc.

use super::rustc_driver;
use super::rustc_error_messages;
use super::rustc_errors;
use super::rustc_span;

use std::{
    mem,
    sync::{Arc, Mutex},
};

use rustc_error_messages::DiagMessage;
use rustc_errors::{registry::Registry, DiagCtxt, LazyFallbackBundle};
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

    /// Returns a `rustc_errors::DiagCtxt` which stores emitted errors into this accumulator.
    ///
    /// The `DiagCtxt` ignores non-error diagnostics.
    pub fn into_diag_ctxt(self) -> DiagCtxt {
        DiagCtxt::new(Box::new(ErrorEmitter::new(self)))
    }

    /// Adds a non-rustc error message to the accumulator.
    pub fn push(&self, msg: String) {
        self.contents.lock().unwrap().push(msg);
    }
}

struct ErrorEmitter {
    fallback_bundle: LazyFallbackBundle,
    accumulator: ErrorAccumulator,
}

impl ErrorEmitter {
    fn new(error_list: ErrorAccumulator) -> Self {
        let fallback_bundle = rustc_errors::fallback_fluent_bundle(
            rustc_driver::DEFAULT_LOCALE_RESOURCES.to_vec(),
            false,
        );
        ErrorEmitter {
            fallback_bundle,
            accumulator: error_list,
        }
    }
}

impl rustc_errors::translation::Translate for ErrorEmitter {
    fn fluent_bundle(&self) -> Option<&rustc_errors::FluentBundle> {
        None
    }

    fn fallback_fluent_bundle(&self) -> &rustc_errors::FluentBundle {
        &self.fallback_bundle
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
}
