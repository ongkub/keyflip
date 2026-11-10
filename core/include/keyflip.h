#ifndef KEYFLIP_H
#define KEYFLIP_H
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Each engine is SINGLE-THREADED: use/free it only on its owning thread.
 * Non-null pointers must be valid; strings must be NUL-terminated. Null handles
 * and invalid UTF-8 return pass/NULL (void operations ignore invalid input).
 * Never pass freed handles/strings. Rust panics are contained at the ABI.
 * Config: detector snake_case keys plus deleteMode (scalar/grapheme) and never
 * (array of strings; initialNever/neverList also accepted). Pairs default to Mac.
 * Grapheme mode groups Thai/Unicode combining marks, approximating Thai/Latin
 * characters; it does not implement full Unicode grapheme segmentation.
 */
typedef struct KFEngine KFEngine;
typedef struct {
    uint32_t kind;          /* 0 pass, 1 replace, 2 revert */
    uint32_t delete_count;
    const char *insert;     /* UTF-8, engine-owned; valid until next kf_key/free.
                            * NULL for pass; do not kf_string_free this pointer. */
    uint32_t switch_to;     /* 0 th, 1 en; ignored for pass */
} KFAction;

KFEngine *kf_engine_new(const char *models_dir, const char *config_json /* nullable */,
                       const char *pairs_json /* nullable */, char **error_out /* nullable; free with kf_string_free */);
void kf_engine_free(KFEngine *e);
KFAction kf_key(KFEngine *e, uint32_t key_kind /* 0 char,1 space,2 tab,3 return,4 backspace,5 undo,6 reset */,
                const char *text /* UTF-8 for key_kind 0, else ignored */);
/* Returned JSON strings are caller-owned; free with kf_string_free.
 * Event objects have event="runEnded"/"converted"/"reverted". runEnded has lang;
 * converted has kind,from,to,score,key (integer or null),prev (th/en or null),
 * word,result; reverted has kind,from,to,word,result. Languages are th/en,
 * or neutral for runEnded. kind is typing/space/backspace/shift.
 */
char *kf_take_events(KFEngine *e); /* JSON array since last call (drains queue) */
char *kf_never_get(KFEngine *e);   /* JSON array of strings, sorted */
void kf_never_set(KFEngine *e, const char *json_array); /* invalid JSON ignored */
void kf_never_clear(KFEngine *e);
void kf_string_free(char *s);
const char *kf_version(void); /* static string; do not free */

#ifdef __cplusplus
}
#endif
#endif
