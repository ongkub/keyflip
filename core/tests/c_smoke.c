#include "keyflip.h"
#include <assert.h>
#include <stddef.h>
#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    assert(argc == 2);
    assert(offsetof(KFAction, kind) == 0);
    assert(offsetof(KFAction, delete_count) == 4);
    assert(offsetof(KFAction, insert) == 8);
    assert(offsetof(KFAction, switch_to) == 8 + sizeof(const char *));
    char *error = NULL;
    KFEngine *e = kf_engine_new(argv[1],
        "{\"threshold\":-1000,\"final_threshold\":-1000,\"min_len\":2,"
        "\"orthography_auto\":false,\"orthography_keep\":0}",
        "[[\"a\",\"ก\"],[\"b\",\"ข\"]]", &error);
    if (!e) {
        fprintf(stderr, "engine: %s\n", error ? error : "unknown error");
        kf_string_free(error);
        return 1;
    }
    assert(error == NULL);
    assert(strcmp(kf_version(), "0.1.0") == 0);
    assert(kf_key(e, 0, "a").kind == 0);
    KFAction a = kf_key(e, 0, "b");
    assert(a.kind == 1 && a.delete_count == 1 && a.switch_to == 0);
    assert(a.insert && strcmp(a.insert, "กข") == 0);
    char *events = kf_take_events(e);
    assert(events && events[0] == '[');
    assert(strstr(events, "\"event\":\"converted\"") != NULL);
    assert(strstr(events, "\"kind\":\"typing\"") != NULL);
    assert(strstr(events, "\"word\":\"ab\"") != NULL);
    assert(strstr(events, "\"result\":\"กข\"") != NULL);
    puts(events);
    kf_string_free(events);
    assert(strcmp(a.insert, "กข") == 0); /* events do not invalidate insert */
    a = kf_key(e, 5, NULL);
    assert(a.kind == 2 && a.delete_count == 2 && a.switch_to == 1);
    assert(strcmp(a.insert, "ab") == 0);
    char *never = kf_never_get(e);
    assert(never && strcmp(never, "[\"ab\"]") == 0);
    kf_string_free(never);
    kf_never_clear(e);
    kf_engine_free(e);
    assert(kf_key(NULL, 0, NULL).kind == 0);
    kf_engine_free(NULL);
    kf_string_free(NULL);
    puts("C ABI smoke passed");
    return 0;
}
