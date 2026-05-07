(function () {
    var instances = {};

    function flushNals(inst, type) {
        if (inst.nalBuf.length === 0 || !inst.decoder) return;

        var total = inst.nalBuf.reduce(function (s, a) { return s + a.byteLength; }, 0);
        var combined = new Uint8Array(total);
        var off = 0;
        inst.nalBuf.forEach(function (a) { combined.set(new Uint8Array(a), off); off += a.byteLength; });
        inst.nalBuf = [];

        try {
            inst.decoder.decode(new EncodedVideoChunk({
                type:      type,
                timestamp: inst.pts,
                data:      combined,
            }));
            inst.pts += 33333; // ~30 fps in µs
        } catch (e) {
            console.warn('wgRD: decode error', e);
        }
    }

    window.wgRemoteDesktop = {
        open: function (canvasId, wsUrl, width, height) {
            var canvas = document.getElementById(canvasId);
            if (!canvas) { return; }

            canvas.width  = width;
            canvas.height = height;
            canvas.setAttribute('tabindex', '0');

            var ctx = canvas.getContext('2d');

            var inst = { nalBuf: [], pts: 0, decoder: null, ws: null };
            instances[canvasId] = inst;

            function initDecoder() {
                if (inst.decoder) { try { inst.decoder.close(); } catch (e) {} inst.decoder = null; }
                inst.decoder = new VideoDecoder({
                    output: function (frame) {
                        ctx.drawImage(frame, 0, 0, canvas.width, canvas.height);
                        frame.close();
                    },
                    error: function (e) { console.error('wgRD: VideoDecoder error', e); },
                });
                inst.decoder.configure({
                    codec:  'avc1.42001f', // H.264 Baseline Profile Level 3.1
                    codedWidth:         width,
                    codedHeight:        height,
                    optimizeForLatency: true,
                });
            }

            var ws = new WebSocket(wsUrl);
            inst.ws = ws;
            ws.binaryType = 'arraybuffer';

            ws.addEventListener('open', function () { initDecoder(); canvas.focus(); });

            ws.addEventListener('message', function (ev) {
                if (!(ev.data instanceof ArrayBuffer) || !inst.decoder) { return; }

                var data = new Uint8Array(ev.data);
                // Skip Annex B start code to find the NAL header byte
                var skip = 0;
                if (data.length > 4 && data[0] === 0 && data[1] === 0) {
                    skip = (data[2] === 1) ? 3 : (data[2] === 0 && data[3] === 1) ? 4 : 0;
                }
                var nalType = (data[skip] & 0x1f);

                if (nalType === 7) {          // SPS — start of key frame sequence
                    flushNals(inst, 'delta'); // flush any dangling delta NAL
                    inst.nalBuf.push(ev.data);
                } else if (nalType === 8) {   // PPS — accumulate
                    inst.nalBuf.push(ev.data);
                } else if (nalType === 5) {   // IDR slice — complete key frame
                    inst.nalBuf.push(ev.data);
                    flushNals(inst, 'key');
                } else {                      // Non-IDR (type 1 etc.) — delta frame
                    flushNals(inst, 'delta');
                    inst.nalBuf.push(ev.data);
                    flushNals(inst, 'delta');
                }
            });

            ws.addEventListener('close', function () {
                if (inst.decoder) { try { inst.decoder.close(); } catch (e) {} inst.decoder = null; }
                ctx.fillStyle = '#111';
                ctx.fillRect(0, 0, canvas.width, canvas.height);
                ctx.fillStyle = '#888';
                ctx.font = '16px monospace';
                ctx.fillText('[disconnected]', 16, 32);
            });
            ws.addEventListener('error', function (e) { console.error('wgRD: ws error', e); });

            function sendEvent(obj) {
                if (inst.ws && inst.ws.readyState === WebSocket.OPEN) {
                    inst.ws.send(JSON.stringify(obj));
                }
            }

            canvas.addEventListener('mousemove', function (e) {
                var r = canvas.getBoundingClientRect();
                var scaleX = width  / (r.width  || 1);
                var scaleY = height / (r.height || 1);
                sendEvent({ t: 'mouse_move',
                    x: Math.round((e.clientX - r.left) * scaleX),
                    y: Math.round((e.clientY - r.top)  * scaleY),
                });
            });
            canvas.addEventListener('mousedown',   function (e) { sendEvent({ t: 'mouse_down',   btn: e.button }); e.preventDefault(); });
            canvas.addEventListener('mouseup',     function (e) { sendEvent({ t: 'mouse_up',     btn: e.button }); });
            canvas.addEventListener('wheel',       function (e) {
                sendEvent({ t: 'mouse_scroll', dx: Math.round(e.deltaX), dy: Math.round(e.deltaY) });
                e.preventDefault();
            }, { passive: false });
            canvas.addEventListener('keydown',     function (e) { sendEvent({ t: 'key_down', code: e.code }); e.preventDefault(); });
            canvas.addEventListener('keyup',       function (e) { sendEvent({ t: 'key_up',   code: e.code }); });
            canvas.addEventListener('contextmenu', function (e) { e.preventDefault(); });
        },

        dispose: function (canvasId) {
            var inst = instances[canvasId];
            if (!inst) { return; }
            if (inst.decoder) { try { inst.decoder.close(); } catch (e) {} }
            if (inst.ws)      { try { inst.ws.close();      } catch (e) {} }
            delete instances[canvasId];
        },

        sendPli: function (canvasId) {
            var inst = instances[canvasId];
            if (inst && inst.ws && inst.ws.readyState === WebSocket.OPEN) {
                inst.ws.send(JSON.stringify({ t: 'pli' }));
            }
        },
    };
})();
