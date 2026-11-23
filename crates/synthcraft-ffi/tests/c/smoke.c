/* Plain C smoke test for the SynthCraft ABI. Build and run:
 *   cargo build --release -p synthcraft-ffi
 *   cc tests/c/smoke.c -Iinclude target/release/libsynthcraft_ffi.a -ldl -lpthread -lm -o smoke && ./smoke
 */
#include "synthcraft.h"
#include <math.h>
#include <stdio.h>
#include <string.h>

static int failures = 0;
#define CHECK(c) do { if (!(c)) { printf("FAIL line %d: %s\n", __LINE__, #c); failures++; } } while (0)

static double energy(const float *b, unsigned n) {
  double e = 0;
  for (unsigned i = 0; i < n; i++) e += (double)b[i] * b[i];
  return e;
}

int main(void) {
  CHECK(sc_abi_version() == SC_ABI_VERSION);
  CHECK(sc_param_count() >= 22);
  sc_engine *e = sc_create(48000.0f);
  CHECK(e != NULL);

  static float l[4800], r[4800];
  sc_process(e, l, r, 4800);
  CHECK(energy(l, 4800) < 1e-9);

  sc_set_max_voices(e, 4);
  sc_note_on(e, 60, 0.9f);
  sc_note_on(e, 64, 0.9f);
  sc_process(e, l, r, 4800);
  CHECK(sc_active_voices(e) == 2);
  CHECK(energy(l, 4800) > 0.01);
  CHECK(memcmp(l, r, sizeof l) == 0);
  for (unsigned i = 0; i < 4800; i++) CHECK(isfinite(l[i]) && fabsf(l[i]) <= 1.5f);

  sc_param_info info;
  for (unsigned i = 0; i < sc_param_count(); i++) {
    CHECK(sc_param_info_at(i, &info) == 1);
    CHECK(info.key[0] != 0 && info.name[0] != 0);
    float mid = 0.5f * (info.min + info.max);
    CHECK(sc_set_param(e, info.id, mid) == 1);
    CHECK(sc_get_param(e, info.id) == mid);
  }
  CHECK(sc_set_param(e, 0xFFFF, 1.0f) == 0);

  char *state = sc_state_json(e);
  CHECK(state != NULL);
  sc_engine *e2 = sc_create(44100.0f);
  CHECK(sc_load_state_json(e2, state) == 0);
  sc_free_string(state);
  CHECK(sc_load_state_json(e2, "{broken") == 1);
  CHECK(strlen(sc_last_error(e2)) > 0);
  CHECK(sc_load_bank(e2, "/no/such/bank.sfz") == 1);

  sc_note_off(e, 60);
  sc_note_off(e, 64);
  sc_all_notes_off(e);
  sc_set_sample_rate(e, 96000.0f);
  sc_destroy(e);
  sc_destroy(e2);
  sc_destroy(NULL);

  printf(failures ? "C smoke: %d failure(s)\n" : "C smoke: all checks passed\n", failures);
  return failures != 0;
}
