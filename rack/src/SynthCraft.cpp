// SynthCraft for VCV Rack: a polyphonic synthesizer module on the SynthCraft engine.
//
// The DSP is the same Rust engine the desktop app and CLAP plugin use, reached through the C
// ABI in synthcraft.h. This file only maps Rack's cables, knobs and patch state onto it.
//
// Inputs:  V/OCT (poly, 0 V = C4), GATE (poly), VEL (poly, 0..10 V), CUTOFF CV (1 V/oct),
//          RESO CV, MORPH CV, PWM CV (0..10 V spans the full parameter range; mono, channel 1)
// Outputs: LEFT, RIGHT (the engine is mono today, so both carry the same signal)

#include "plugin.hpp"
#include "synthcraft.h"
#include <osdialog.h>
#include <atomic>
#include <cmath>
#include <mutex>
#include <string>

namespace {

constexpr int N_PARAMS = 25;
constexpr int BLOCK = 16;
constexpr int MAX_POLY = 16;

// Short panel labels, in the order of the engine's parameter table.
const char* const LABELS[N_PARAMS] = {
	"DETUNE", "OSC MIX", "FM",      "CUTOFF", "RESO",   "ENV AMT", "ATTACK", "DECAY",
	"SUSTAIN", "LFO HZ", "RELEASE", "LFO AMT", "DRIVE", "LEVEL",   "PW",     "PCM MIX",
	"MORPH",  "SAMPLE", "CHORUS",  "REVERB", "REV TIME", "REV DAMP",
	"SPREAD", "AMB AMT", "AMB HZ",
};

// Parameters whose knob is exponential (the knob value is log2 of the engine value).
bool isLogParam(const std::string& key) {
	return key == "cutoff_hz" || key == "attack_ms" || key == "decay_ms" || key == "release_ms" ||
	       key == "lfo_rate_hz";
}

const char* unitOf(const std::string& key) {
	if (key == "cutoff_hz" || key == "lfo_rate_hz" || key == "ambient_rate_hz") return " Hz";
	if (key.size() > 3 && key.compare(key.size() - 3, 3, "_ms") == 0) return " ms";
	if (key == "vco2_detune_cents") return " cents";
	if (key == "reverb_decay_s") return " s";
	return "";
}

}  // namespace

struct SynthCraft : Module {
	enum InputId {
		VOCT_INPUT,
		GATE_INPUT,
		VEL_INPUT,
		CUTOFF_CV_INPUT,
		RESO_CV_INPUT,
		MORPH_CV_INPUT,
		PWM_CV_INPUT,
		INPUTS_LEN
	};
	enum OutputId { LEFT_OUTPUT, RIGHT_OUTPUT, OUTPUTS_LEN };

	sc_engine* eng = nullptr;
	std::mutex engMutex;  // the audio thread only ever try-locks it
	sc_param_info info[N_PARAMS];
	bool isLog[N_PARAMS] = {};
	bool ready = false;

	bool gateHigh[MAX_POLY] = {};
	int note[MAX_POLY] = {};
	float bufL[BLOCK] = {};
	float bufR[BLOCK] = {};
	int pos = BLOCK;
	int maxVoicesIdx = 3;  // index into {1,2,4,8,16}
	std::string statusText;

	static constexpr int VOICE_CHOICES[5] = {1, 2, 4, 8, 16};
	static constexpr const char* WAVE_KEYS[10] = {
		"sine", "saw", "square", "triangle", "pulse_pwm",
		"supersaw", "white_noise", "sub_octave", "atmo_pad", "wavetable",
	};
	static constexpr const char* FILTER_KEYS[4] = {
		"lowpass_24db", "bandpass_12db", "highpass_12db", "acid_resonant",
	};
	static constexpr const char* WAVETABLE_KEYS[4] = {
		"analog_morph", "formant_vowel", "metallic_bell", "acid_fold",
	};

	SynthCraft() {
		config(N_PARAMS, INPUTS_LEN, OUTPUTS_LEN, 0);
		if (sc_abi_version() != SC_ABI_VERSION || (int) sc_param_count() != N_PARAMS) {
			WARN("SynthCraft: engine library does not match this plugin (abi %u, %u params)",
			     sc_abi_version(), sc_param_count());
			statusText = "engine library mismatch";
			return;
		}
		for (int i = 0; i < N_PARAMS; i++) {
			sc_param_info_at(i, &info[i]);
			std::string key = info[i].key;
			isLog[i] = isLogParam(key);
			std::string name = info[i].name;
			if (isLog[i]) {
				configParam(i, std::log2(info[i].min), std::log2(info[i].max),
				            std::log2(info[i].default_value), name, unitOf(key), 2.f, 1.f);
			}
			else {
				configParam(i, info[i].min, info[i].max, info[i].default_value, name, unitOf(key));
			}
		}
		configInput(VOCT_INPUT, "V/Oct (polyphonic)");
		configInput(GATE_INPUT, "Gate (polyphonic)");
		configInput(VEL_INPUT, "Velocity (polyphonic, 0 to 10 V)");
		configInput(CUTOFF_CV_INPUT, "Cutoff CV (1 V/oct)");
		configInput(RESO_CV_INPUT, "Resonance CV");
		configInput(MORPH_CV_INPUT, "Morph CV");
		configInput(PWM_CV_INPUT, "Pulse width CV");
		configOutput(LEFT_OUTPUT, "Left");
		configOutput(RIGHT_OUTPUT, "Right");

		float sr = (APP && APP->engine) ? APP->engine->getSampleRate() : 44100.f;
		eng = sc_create(sr);
		if (!eng) {
			statusText = "could not create engine";
			return;
		}
		sc_set_max_voices(eng, VOICE_CHOICES[maxVoicesIdx]);
		ready = true;
	}

	~SynthCraft() {
		std::lock_guard<std::mutex> lock(engMutex);
		sc_destroy(eng);
		eng = nullptr;
		ready = false;
	}

	float engineValue(int i) {
		float v = params[i].getValue();
		return isLog[i] ? std::exp2(v) : v;
	}

	// Push knob and CV values into the engine once per block.
	void updateParams() {
		float cutoffCv = inputs[CUTOFF_CV_INPUT].getVoltage();
		float resoCv = inputs[RESO_CV_INPUT].getVoltage();
		float morphCv = inputs[MORPH_CV_INPUT].getVoltage();
		float pwmCv = inputs[PWM_CV_INPUT].getVoltage();
		for (int i = 0; i < N_PARAMS; i++) {
			float v = engineValue(i);
			std::string key = info[i].key;
			float span = info[i].max - info[i].min;
			if (key == "cutoff_hz") v *= std::exp2(cutoffCv);
			else if (key == "resonance_q") v += resoCv * 0.1f * span;
			else if (key == "extreme_morph") v += morphCv * 0.1f * span;
			else if (key == "pulse_width") v += pwmCv * 0.1f * span;
			sc_set_param(eng, info[i].id, v);
		}
	}

	static int noteFromVolts(float v) {
		int n = (int) std::lround(60.f + 12.f * v);
		return clamp(n, 0, 127);
	}

	void handleGates() {
		int channels = std::max(inputs[GATE_INPUT].getChannels(), inputs[VOCT_INPUT].getChannels());
		channels = std::min(channels, MAX_POLY);
		for (int c = 0; c < channels; c++) {
			float g = inputs[GATE_INPUT].getPolyVoltage(c);
			bool high = gateHigh[c] ? (g > 0.1f) : (g >= 2.f);
			int n = noteFromVolts(inputs[VOCT_INPUT].getPolyVoltage(c));
			if (high && !gateHigh[c]) {
				float vel = inputs[VEL_INPUT].isConnected()
				                ? clamp(inputs[VEL_INPUT].getPolyVoltage(c) / 10.f, 0.05f, 1.f)
				                : 0.8f;
				note[c] = n;
				sc_note_on(eng, (uint8_t) n, vel);
			}
			else if (high && gateHigh[c] && n != note[c]) {
				// Pitch moved while the gate stays high: retrigger on the new note.
				sc_note_off(eng, (uint8_t) note[c]);
				float vel = inputs[VEL_INPUT].isConnected()
				                ? clamp(inputs[VEL_INPUT].getPolyVoltage(c) / 10.f, 0.05f, 1.f)
				                : 0.8f;
				note[c] = n;
				sc_note_on(eng, (uint8_t) n, vel);
			}
			else if (!high && gateHigh[c]) {
				sc_note_off(eng, (uint8_t) note[c]);
			}
			gateHigh[c] = high;
		}
		// Channels that disappeared (cable pulled or fewer poly channels) release their notes.
		for (int c = channels; c < MAX_POLY; c++) {
			if (gateHigh[c]) {
				sc_note_off(eng, (uint8_t) note[c]);
				gateHigh[c] = false;
			}
		}
	}

	void process(const ProcessArgs& args) override {
		outputs[LEFT_OUTPUT].setChannels(1);
		outputs[RIGHT_OUTPUT].setChannels(1);
		std::unique_lock<std::mutex> lock(engMutex, std::try_to_lock);
		if (!ready || !lock.owns_lock()) {
			// Engine busy (bank or state loading on the UI thread): one silent sample.
			outputs[LEFT_OUTPUT].setVoltage(0.f);
			outputs[RIGHT_OUTPUT].setVoltage(0.f);
			return;
		}
		handleGates();
		if (pos >= BLOCK) {
			updateParams();
			sc_process(eng, bufL, bufR, BLOCK);
			pos = 0;
		}
		// Engine output is +-1; Rack's audio level is +-5 V.
		outputs[LEFT_OUTPUT].setVoltage(5.f * bufL[pos]);
		outputs[RIGHT_OUTPUT].setVoltage(5.f * bufR[pos]);
		pos++;
	}

	void onSampleRateChange(const SampleRateChangeEvent& e) override {
		if (!ready) return;
		std::lock_guard<std::mutex> lock(engMutex);
		sc_set_sample_rate(eng, e.sampleRate);
		for (int c = 0; c < MAX_POLY; c++) gateHigh[c] = false;
		pos = BLOCK;
	}

	void onReset(const ResetEvent& e) override {
		Module::onReset(e);
		if (!ready) return;
		std::lock_guard<std::mutex> lock(engMutex);
		sc_all_notes_off(eng);
		for (int c = 0; c < MAX_POLY; c++) gateHigh[c] = false;
	}

	void setMaxVoices(int idx) {
		idx = clamp(idx, 0, 4);
		if (!ready) return;
		std::lock_guard<std::mutex> lock(engMutex);
		maxVoicesIdx = idx;
		sc_set_max_voices(eng, VOICE_CHOICES[idx]);
	}

	// Menu actions run on the UI thread and hold the lock while loading, so the audio thread
	// outputs silence for that moment instead of touching a half-loaded engine.
	void loadBank(const std::string& path) {
		if (!ready) return;
		std::lock_guard<std::mutex> lock(engMutex);
		if (sc_load_bank(eng, path.c_str()) == 0) {
			statusText = "bank: " + path;
		}
		else {
			statusText = std::string("bank failed: ") + sc_last_error(eng);
			WARN("SynthCraft: %s", statusText.c_str());
		}
	}

	void clearBank() {
		if (!ready) return;
		std::lock_guard<std::mutex> lock(engMutex);
		sc_clear_bank(eng);
		statusText = "no bank";
	}

	// Read a top-level string field from the engine's current patch JSON.
	std::string getPatchString(const char* key) {
		if (!ready) return {};
		std::lock_guard<std::mutex> lock(engMutex);
		char* raw = sc_state_json(eng);
		if (!raw) return {};
		json_error_t err;
		json_t* root = json_loads(raw, 0, &err);
		sc_free_string(raw);
		if (!root) return {};
		std::string val;
		if (json_t* f = json_object_get(root, key); f && json_is_string(f)) {
			val = json_string_value(f);
		}
		json_decref(root);
		return val;
	}

	// Update a top-level string field in the engine's patch JSON (zero C ABI break).
	void setPatchString(const char* key, const char* value) {
		if (!ready) return;
		std::lock_guard<std::mutex> lock(engMutex);
		char* raw = sc_state_json(eng);
		if (!raw) return;
		json_error_t err;
		json_t* root = json_loads(raw, 0, &err);
		sc_free_string(raw);
		if (!root) return;
		json_object_set_new(root, key, json_string(value));
		char* dumped = json_dumps(root, JSON_COMPACT);
		json_decref(root);
		if (!dumped) return;
		int rc = sc_load_state_json(eng, dumped);
		std::free(dumped);
		if (rc == 0) {
			statusText = std::string(key) + ": " + value;
		}
		else {
			statusText = std::string("patch update failed: ") + sc_last_error(eng);
			WARN("SynthCraft: %s", statusText.c_str());
		}
		pos = BLOCK;
	}

	size_t findIndex(const std::string& val, const char* const* table, size_t n) const {
		for (size_t i = 0; i < n; i++) {
			if (val == table[i]) return i;
		}
		return 0;
	}

	json_t* dataToJson() override {
		json_t* root = json_object();
		json_object_set_new(root, "maxVoicesIdx", json_integer(maxVoicesIdx));
		if (ready) {
			std::lock_guard<std::mutex> lock(engMutex);
			char* state = sc_state_json(eng);
			if (state) {
				json_object_set_new(root, "patch", json_string(state));
				sc_free_string(state);
			}
		}
		return root;
	}

	void dataFromJson(json_t* root) override {
		if (!ready) return;
		if (json_t* v = json_object_get(root, "maxVoicesIdx")) setMaxVoices((int) json_integer_value(v));
		json_t* p = json_object_get(root, "patch");
		if (!p || !json_is_string(p)) return;
		std::lock_guard<std::mutex> lock(engMutex);
		int rc = sc_load_state_json(eng, json_string_value(p));
		if (rc == 1) {
			statusText = std::string("patch not loaded: ") + sc_last_error(eng);
			WARN("SynthCraft: %s", statusText.c_str());
		}
		else if (rc == 2) {
			statusText = sc_last_error(eng);
			WARN("SynthCraft: %s", statusText.c_str());
		}
		pos = BLOCK;
	}
};

constexpr int SynthCraft::VOICE_CHOICES[5];
constexpr const char* SynthCraft::WAVE_KEYS[10];
constexpr const char* SynthCraft::FILTER_KEYS[4];
constexpr const char* SynthCraft::WAVETABLE_KEYS[4];

namespace {

struct TextLabel : widget::TransparentWidget {
	std::string text;
	float size = 8.f;
	void draw(const DrawArgs& args) override {
		std::shared_ptr<window::Font> font =
		    APP->window->loadFont(asset::system("res/fonts/ShareTechMono-Regular.ttf"));
		if (!font) return;
		nvgFontFaceId(args.vg, font->handle);
		nvgFontSize(args.vg, size);
		nvgFillColor(args.vg, nvgRGB(0xd8, 0xdc, 0xe6));
		nvgTextAlign(args.vg, NVG_ALIGN_CENTER | NVG_ALIGN_MIDDLE);
		nvgText(args.vg, box.size.x / 2.f, box.size.y / 2.f, text.c_str(), NULL);
	}
};

TextLabel* makeLabel(Vec centerMm, const std::string& text, float size, float widthMm = 17.f) {
	TextLabel* l = new TextLabel;
	l->text = text;
	l->size = size;
	l->box.size = mm2px(Vec(widthMm, 4.f));
	l->box.pos = mm2px(centerMm).minus(l->box.size.div(2.f));
	return l;
}

}  // namespace

struct SynthCraftWidget : ModuleWidget {
	SynthCraftWidget(SynthCraft* module) {
		setModule(module);
		setPanel(createPanel(asset::plugin(pluginInstance, "res/SynthCraft.svg")));

		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(box.size.x - 2 * RACK_GRID_WIDTH, 0)));
		addChild(createWidget<ScrewSilver>(Vec(RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));
		addChild(createWidget<ScrewSilver>(
		    Vec(box.size.x - 2 * RACK_GRID_WIDTH, RACK_GRID_HEIGHT - RACK_GRID_WIDTH)));

		addChild(makeLabel(Vec(50.8f, 4.f), "SYNTHCRAFT", 11.f, 60.f));

		// 25 knobs in a grid of 5 columns.
		for (int i = 0; i < N_PARAMS; i++) {
			int col = i % 5;
			int row = i / 5;
			float x = 12.f + col * 19.f;
			float y = 20.f + row * 17.5f;
			addParam(createParamCentered<RoundSmallBlackKnob>(mm2px(Vec(x, y)), module, i));
			addChild(makeLabel(Vec(x, y + 7.2f), LABELS[i], 6.5f));
		}

		// Inputs and outputs along the bottom.
		const char* inLabels[SynthCraft::INPUTS_LEN] = {"V/OCT", "GATE", "VEL", "CUTOFF", "RESO", "MORPH", "PWM"};
		for (int i = 0; i < SynthCraft::INPUTS_LEN; i++) {
			float x = 9.f + i * 11.6f;
			addInput(createInputCentered<PJ301MPort>(mm2px(Vec(x, 112.f)), module, i));
			addChild(makeLabel(Vec(x, 118.6f), inLabels[i], 6.f, 11.f));
		}
		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(9.f + 7.5f * 11.6f, 112.f)), module,
		                                           SynthCraft::LEFT_OUTPUT));
		addChild(makeLabel(Vec(9.f + 7.5f * 11.6f, 118.6f), "L", 6.f, 11.f));
		addOutput(createOutputCentered<PJ301MPort>(mm2px(Vec(9.f + 8.5f * 11.6f, 112.f)), module,
		                                           SynthCraft::RIGHT_OUTPUT));
		addChild(makeLabel(Vec(9.f + 8.5f * 11.6f, 118.6f), "R", 6.f, 11.f));
	}

	void appendContextMenu(Menu* menu) override {
		SynthCraft* m = dynamic_cast<SynthCraft*>(module);
		if (!m) return;
		menu->addChild(new MenuSeparator);
		menu->addChild(createIndexSubmenuItem(
		    "Max voices", {"1", "2", "4", "8", "16"}, [=]() { return (size_t) m->maxVoicesIdx; },
		    [=](size_t i) { m->setMaxVoices((int) i); }));
		const std::vector<std::string> waveLabels = {
			"Sine", "Saw", "Square", "Triangle", "Pulse PWM",
			"SuperSaw", "White Noise", "Sub Octave", "Atmo Pad", "Wavetable",
		};
		menu->addChild(createIndexSubmenuItem(
		    "VCO 1 waveform", waveLabels,
		    [=]() { return m->findIndex(m->getPatchString("vco1_wave"), SynthCraft::WAVE_KEYS, 10); },
		    [=](size_t i) {
			    if (i < 10) m->setPatchString("vco1_wave", SynthCraft::WAVE_KEYS[i]);
		    }));
		menu->addChild(createIndexSubmenuItem(
		    "VCO 2 waveform", waveLabels,
		    [=]() { return m->findIndex(m->getPatchString("vco2_wave"), SynthCraft::WAVE_KEYS, 10); },
		    [=](size_t i) {
			    if (i < 10) m->setPatchString("vco2_wave", SynthCraft::WAVE_KEYS[i]);
		    }));
		menu->addChild(createIndexSubmenuItem(
		    "Filter mode", {"LowPass 24dB", "BandPass 12dB", "HighPass 12dB", "Acid Resonant"},
		    [=]() { return m->findIndex(m->getPatchString("filter_mode"), SynthCraft::FILTER_KEYS, 4); },
		    [=](size_t i) {
			    if (i < 4) m->setPatchString("filter_mode", SynthCraft::FILTER_KEYS[i]);
		    }));
		menu->addChild(createIndexSubmenuItem(
		    "Wavetable", {"Analog Morph", "Formant Vowel", "Metallic Bell", "Acid Fold"},
		    [=]() { return m->findIndex(m->getPatchString("wavetable"), SynthCraft::WAVETABLE_KEYS, 4); },
		    [=](size_t i) {
			    if (i < 4) m->setPatchString("wavetable", SynthCraft::WAVETABLE_KEYS[i]);
		    }));
		menu->addChild(createMenuItem("Load sample bank (.sfz .sf2 .wav)...", "", [=]() {
			osdialog_filters* filters = osdialog_filters_parse("Sample bank:sfz,sf2,wav");
			char* path = osdialog_file(OSDIALOG_OPEN, NULL, NULL, filters);
			osdialog_filters_free(filters);
			if (path) {
				m->loadBank(path);
				std::free(path);
			}
		}));
		menu->addChild(createMenuItem("Remove sample bank", "", [=]() { m->clearBank(); }));
		if (!m->statusText.empty()) menu->addChild(createMenuLabel(m->statusText));
	}
};

Model* modelSynthCraft = createModel<SynthCraft, SynthCraftWidget>("SynthCraft");
