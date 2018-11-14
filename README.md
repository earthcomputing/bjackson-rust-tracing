# rust-tracing

## coding idiom (somewhere within listen_xx_loop):

    let _f = "event_loop"; 
    if TRACE_OPTIONS.all || TRACE_OPTIONS.pe {
        let ref code_attr = CodeAttributes { module: file!(), function: _f, line_no: line!(), format: "recv" };
        let ref body = json!({ "id": self.get_id(), ...  });
        let entry = emitter::unhide().trace(code: code_attr, body: body);
        let _ = dal::add_trace(entry);
    }

    // SPAWN THREAD (listen_xx_loop)
    fn worker(&self, ...) -> Result<(), Err> {
        let _f = "worker";
        let mut o = self.clone(); // or new ...
        let child_emitter = emitter::unhide().fork_trace();
        let thread_name = format!("xx {} event_loop", self.get_id());
        let h = thread::Builder::new().name(thread_name.into()).spawn( move || {
            let ref mut working_emitter = child_emitter.clone();
            emitter::stash(working_emitter);
            let _ = o.event_loop(...).map_err(|e| write_err("xx", e));
            // if CONTINUE_ON_ERROR { let _ = xx.listen_xx(...); }
        });
        h
    }
