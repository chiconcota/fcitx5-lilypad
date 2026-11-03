#ifndef BAMBOO_CORE_H_WRAPPER
#define BAMBOO_CORE_H_WRAPPER

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef const char cchar;

typedef struct {
    bool autoNonVnRestore;
    bool ddFreeStyle;
    bool macroEnabled;
    bool autoCapitalizeMacro;
    bool spellCheckWithDicts;
    const char *outputCharset;
    bool modernStyle;
    bool freeMarking;
    int w2u;
    int bracketTransform;
    const char *timeFormat;
    const char *dateFormat;
} FcitxBambooEngineOption;

void Init(void);
uint8_t EngineProcessKeyEvent(size_t engine, uint32_t keyVal, uint32_t state);
void EngineSetRestoreKeyStroke(size_t engine);
char* EnginePullPreedit(size_t engine);
void EngineCommitPreedit(size_t engine);
char* EnginePullCommit(size_t engine);
void EngineSetOption(size_t engine, FcitxBambooEngineOption* option);
size_t NewEngine(const char* name, size_t dictHandle, size_t tableHandle);
size_t NewCustomEngine(char** definition, size_t dictHandle, size_t tableHandle);
size_t NewMacroTable(char** definition);
void DeleteObject(size_t handle);
void ResetEngine(size_t engine);
void EngineRebuildFromText(size_t engine, const char* text);
char** GetCharsetNames(void);
char** GetInputMethodNames(void);
size_t NewDictionary(size_t fd);

#ifdef __cplusplus
}
#endif

#endif // BAMBOO_CORE_H_WRAPPER
