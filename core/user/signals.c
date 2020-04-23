#include "signals.h"

static void catch_sig_term(int _) {
    writestr(STDOUT_FILENO, "SIG_TERM caught");
    exit(EXIT_SUCCESS);
}

static void catch_sig_quit(int _) {
    writestr(STDOUT_FILENO, "SIG_QUIT caught");
    exit(EXIT_SUCCESS);
}

void main_signals() {

    signal(SIG_TERM, catch_sig_term);
    signal(SIG_QUIT, catch_sig_quit);

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wmissing-noreturn"
    while (1) {
        writestr(STDOUT_FILENO, "SIGNALS");
    }
#pragma clang diagnostic pop

}
