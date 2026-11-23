/* SynthCraft plugin engine, C ABI. Matches crates/synthcraft-ffi/src/lib.rs. */
#ifndef SYNTHCRAFT_H
#define SYNTHCRAFT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define SC_ABI_VERSION 1u

typedef struct sc_engine sc_engine;

typedef struct sc_param_info {
  uint32_t id;
  float min;
  float max;
  float default_value;
  char key[32];
  char name[48];
  char module[16];
} sc_param_info;

uint32_t sc_abi_version(void);

/* Off the audio thread only: create, destroy, set_sample_rate, load/clear bank, state. */
sc_engine *sc_create(float sample_rate);
void sc_destroy(sc_engine *e);
void sc_set_sample_rate(sc_engine *e, float sample_rate);

/* Real-time safe. */
void sc_note_on(sc_engine *e, uint8_t note, float velocity);
void sc_note_off(sc_engine *e, uint8_t note);
void sc_all_notes_off(sc_engine *e);
void sc_pitch_bend(sc_engine *e, float semitones);
void sc_mod_wheel(sc_engine *e, float value);
unsigned sc_active_voices(sc_engine *e);
void sc_set_max_voices(sc_engine *e, unsigned voices);
int sc_set_param(sc_engine *e, unsigned id, float value);
float sc_get_param(sc_engine *e, unsigned id);
void sc_process(sc_engine *e, float *left, float *right, unsigned frames);

unsigned sc_param_count(void);
int sc_param_info_at(unsigned index, sc_param_info *out);
float sc_param_default(unsigned id);

int sc_load_bank(sc_engine *e, const char *path);
void sc_clear_bank(sc_engine *e);
char *sc_state_json(sc_engine *e);
void sc_free_string(char *s);
int sc_load_state_json(sc_engine *e, const char *json);
const char *sc_last_error(sc_engine *e);

#ifdef __cplusplus
}
#endif

#endif
