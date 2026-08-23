#include <stdio.h>
#include "validate.h"

int validate_input(int value) {
#ifdef DISABLE_CHECK
    /* Check disabled at compile time via CFLAGS */
    return 1;
#else
    if (value < 0 || value > 100) {
        return 0;
    }
    return 1;
#endif
}
