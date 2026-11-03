/* C interface for bamboo-core. */
#ifndef BAMBOO_CORE_H
#define BAMBOO_CORE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#if defined(_MSC_VER)
#define BAMBOO_DEPRECATED(message) __declspec(deprecated(message))
#elif defined(__GNUC__) || defined(__clang__)
#define BAMBOO_DEPRECATED(message) __attribute__((deprecated(message)))
#else
#define BAMBOO_DEPRECATED(message)
#endif

typedef struct BambooEngine BambooEngine;

enum BambooMethod {
    BAMBOO_METHOD_TELEX = 0,
    BAMBOO_METHOD_VNI = 1,
    BAMBOO_METHOD_VIQR = 2,
    BAMBOO_METHOD_MICROSOFT_LAYOUT = 3,
    BAMBOO_METHOD_TELEX_2 = 4,
    BAMBOO_METHOD_TELEX_W = 5,
    BAMBOO_METHOD_TELEX_VNI = 6,
    BAMBOO_METHOD_TELEX_VNI_VIQR = 7,
    BAMBOO_METHOD_VNI_FRENCH_LAYOUT = 8,
};

enum BambooMode {
    BAMBOO_MODE_ENGLISH = 0,
    BAMBOO_MODE_VIETNAMESE = 1,
};

enum BambooToneRefresh {
    BAMBOO_TONE_RECOMPUTE = 0,
    BAMBOO_TONE_PRESERVE_SNAPSHOT = 1,
};

enum BambooLastWordForm {
    BAMBOO_LAST_WORD_VIETNAMESE = 0,
    BAMBOO_LAST_WORD_RAW_KEYSTROKES = 1,
};

enum BambooOutputOptions {
    BAMBOO_OUTPUT_NONE = 0,
    BAMBOO_OUTPUT_TONE_LESS = 1 << 0,
    BAMBOO_OUTPUT_MARK_LESS = 1 << 1,
    BAMBOO_OUTPUT_LOWER_CASE = 1 << 2,
    BAMBOO_OUTPUT_FULL_TEXT = 1 << 3,
    BAMBOO_OUTPUT_PUNCTUATION_MODE = 1 << 4,
    BAMBOO_OUTPUT_IN_REVERSE_ORDER = 1 << 5,
    BAMBOO_OUTPUT_RAW = 1 << 6,
};

enum BambooConfigFlags {
    BAMBOO_CONFIG_FREE_TONE_MARKING = 1 << 0,
    BAMBOO_CONFIG_MODERN_TONE_STYLE = 1 << 1,
    BAMBOO_CONFIG_AUTO_CORRECT = 1 << 2,
    BAMBOO_CONFIG_W2U_NON_START = 1 << 3,
    BAMBOO_CONFIG_W2U_EVERYWHERE = 1 << 4,
    BAMBOO_CONFIG_BRACKET_NON_START = 1 << 5,
    BAMBOO_CONFIG_BRACKET_EVERYWHERE = 1 << 6,
};

BAMBOO_DEPRECATED("use BambooEngine instances") void bamboo_setup(void);
BAMBOO_DEPRECATED("use bamboo_engine_reset") void bamboo_reset(void);
BAMBOO_DEPRECATED("use bamboo_engine_set_input_method") void bamboo_set_input_method(int32_t method);
BAMBOO_DEPRECATED("use bamboo_engine_process_key_mode or bamboo_engine_process_key_buf")
char *bamboo_process_key(uint32_t key, int32_t is_vietnamese);
BAMBOO_DEPRECATED("use bamboo_engine_process_key_buf")
int32_t bamboo_process_key_buf(uint32_t key, int32_t is_vietnamese,
                               uint8_t *out_buf, size_t out_cap, size_t *out_len,
                               size_t *backspaces_chars, size_t *backspaces_bytes);
BAMBOO_DEPRECATED("use bamboo_engine_output") char *bamboo_output(void);
BAMBOO_DEPRECATED("use bamboo_engine_undo_last_key") void bamboo_remove_last_char(void);
void bamboo_free_string(char *string);

BambooEngine *bamboo_engine_new(int32_t method);
BambooEngine *bamboo_engine_new_with_flags(int32_t method, uint32_t flags);
void bamboo_engine_free(BambooEngine *engine);
BAMBOO_DEPRECATED("use bamboo_engine_process_key_mode or bamboo_engine_process_key_buf")
char *bamboo_engine_process(BambooEngine *engine, uint32_t key);
char *bamboo_engine_process_key_mode(BambooEngine *engine, uint32_t key, int32_t mode);
int32_t bamboo_engine_process_key_buf(BambooEngine *engine, uint32_t key,
                                      int32_t is_vietnamese, uint8_t *out_buf,
                                      size_t out_cap, size_t *out_len,
                                      size_t *backspaces_chars, size_t *backspaces_bytes);
void bamboo_engine_reset(BambooEngine *engine);
void bamboo_engine_commit(BambooEngine *engine);
void bamboo_engine_set_config_flags(BambooEngine *engine, uint32_t flags);
uint32_t bamboo_engine_config_flags(const BambooEngine *engine);
void bamboo_engine_set_input_method(BambooEngine *engine, int32_t method);
char *bamboo_engine_output(const BambooEngine *engine, uint32_t options);
int32_t bamboo_engine_can_process_key(const BambooEngine *engine, uint32_t key);
int32_t bamboo_engine_is_valid_word(const BambooEngine *engine);
int32_t bamboo_engine_is_valid_prefix(const BambooEngine *engine);
void bamboo_engine_undo_last_key(BambooEngine *engine, int32_t tone_refresh);
void bamboo_engine_remove_last_output_char(BambooEngine *engine);
void bamboo_engine_restore_last_word(BambooEngine *engine, int32_t form);
void bamboo_engine_rebuild_from_text(BambooEngine *engine, const char *text);

#ifdef __cplusplus
}
#endif

#undef BAMBOO_DEPRECATED

#endif
