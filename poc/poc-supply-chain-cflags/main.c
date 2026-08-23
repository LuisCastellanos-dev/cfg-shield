#include <stdio.h>
#include "libvalidate/validate.h"

int main(void) {
    int test_values[] = {-1, 0, 50, 100, 150};
    int n = 5;
    for (int i = 0; i < n; i++) {
        int v = test_values[i];
        printf("validate_input(%4d) = %d\n", v, validate_input(v));
    }
    return 0;
}
