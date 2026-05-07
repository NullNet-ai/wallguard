(function () {
    var instances = {};

    // xterm 5.x UMD exports Terminal directly as window.Terminal.
    // Addon UMD bundles wrap the class: AttachAddon.AttachAddon, FitAddon.FitAddon.
    // Guard against both shapes just in case.
    function getClass(mod) {
        if (typeof mod === 'function') return mod;
        var keys = Object.keys(mod);
        return mod[keys[0]];
    }

    window.wgTerminal = {
        open: function (elementId, wsUrl) {
            var el = document.getElementById(elementId);
            if (!el) { return; }

            if (instances[elementId]) {
                window.wgTerminal.dispose(elementId);
            }

            var term = new Terminal({
                cursorBlink:  true,
                fontFamily:   '"Cascadia Code", "Fira Code", "JetBrains Mono", monospace',
                fontSize:     14,
                convertEol:   true,
                scrollback:   5000,
            });

            var FitAddonClass    = getClass(window.FitAddon);
            var AttachAddonClass = getClass(window.AttachAddon);

            var fitAddon    = new FitAddonClass();
            term.loadAddon(fitAddon);

            var ws = new WebSocket(wsUrl);
            ws.binaryType = 'arraybuffer';
            var attachAddon = new AttachAddonClass(ws);
            term.loadAddon(attachAddon);

            term.open(el);
            fitAddon.fit();
            term.focus();

            ws.addEventListener('close', function () {
                term.write('\r\n\x1b[90m[disconnected]\x1b[0m\r\n');
            });
            ws.addEventListener('error', function () {
                term.write('\r\n\x1b[31m[connection error]\x1b[0m\r\n');
            });

            // Refit when the container resizes.
            var observer = new ResizeObserver(function () { fitAddon.fit(); });
            observer.observe(el);

            instances[elementId] = {
                term:       term,
                ws:         ws,
                fitAddon:   fitAddon,
                attachAddon: attachAddon,
                observer:   observer,
            };
        },

        dispose: function (elementId) {
            var inst = instances[elementId];
            if (!inst) { return; }
            try { inst.observer.disconnect(); }  catch (e) {}
            try { inst.ws.close(); }             catch (e) {}
            try { inst.attachAddon.dispose(); }  catch (e) {}
            try { inst.fitAddon.dispose(); }     catch (e) {}
            try { inst.term.dispose(); }         catch (e) {}
            delete instances[elementId];
        },
    };
})();
