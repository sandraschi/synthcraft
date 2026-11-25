// SynthCraft Web Audio API AudioWorkletProcessor bridge for `synthcraft_wasm.wasm`.
//
// Usage from main thread:
//   await audioCtx.audioWorklet.addModule('synthcraft-worklet.js');
//   const wasmBytes = await (await fetch('synthcraft_wasm.wasm')).arrayBuffer();
//   const node = new AudioWorkletNode(audioCtx, 'synthcraft-processor', {
//     outputChannelCount: [2],
//     processorOptions: { wasmBytes },
//   });
//   node.connect(audioCtx.destination);
//   node.port.postMessage({ type: 'noteOn', note: 60, velocity: 0.9 });

class SynthCraftProcessor extends AudioWorkletProcessor {
  constructor(options) {
    super();
    this.wasm = null;
    this.handle = 0;

    const wasmBytes = options?.processorOptions?.wasmBytes;
    if (wasmBytes) {
      WebAssembly.instantiate(wasmBytes, {}).then(({ instance }) => {
        this.wasm = instance.exports;
        this.handle = this.wasm.scw_create(sampleRate, 128);
        this.port.postMessage({
          type: 'ready',
          patchCount: this.wasm.scw_patch_count(),
          paramCount: this.wasm.scw_param_count(),
        });
      });
    }

    this.port.onmessage = ({ data }) => {
      if (!this.wasm || !this.handle || !data) return;
      switch (data.type) {
        case 'noteOn':
          this.wasm.scw_note_on(this.handle, data.note | 0, Number(data.velocity ?? 0.85));
          break;
        case 'noteOff':
          this.wasm.scw_note_off(this.handle, data.note | 0);
          break;
        case 'allNotesOff':
          this.wasm.scw_all_notes_off(this.handle);
          break;
        case 'play':
          this.wasm.scw_set_playing(this.handle, data.playing ? 1 : 0);
          break;
        case 'loadPatch':
          this.wasm.scw_load_patch_index(this.handle, data.index | 0);
          break;
        case 'setParam':
          this.wasm.scw_set_param(this.handle, data.index | 0, Number(data.value));
          break;
      }
    };
  }

  process(_inputs, outputs) {
    const out = outputs[0];
    if (!this.wasm || !this.handle || !out || out.length === 0) {
      return true;
    }
    const frames = out[0].length;
    const rendered = this.wasm.scw_process(this.handle, frames);
    const mem = this.wasm.memory.buffer;
    const lPtr = this.wasm.scw_left_ptr(this.handle);
    const rPtr = this.wasm.scw_right_ptr(this.handle);
    const leftView = new Float32Array(mem, lPtr, rendered);
    const rightView = new Float32Array(mem, rPtr, rendered);
    out[0].set(leftView);
    if (out.length > 1) {
      out[1].set(rightView);
    }
    return true;
  }
}

registerProcessor('synthcraft-processor', SynthCraftProcessor);
