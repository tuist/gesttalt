#ifndef SHARED_H
#define SHARED_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

char *shared_greeting(const char *name);
int32_t shared_lattice_score(int32_t seed);
void shared_string_free(char *value);

#ifdef __cplusplus
}
#endif

#endif

