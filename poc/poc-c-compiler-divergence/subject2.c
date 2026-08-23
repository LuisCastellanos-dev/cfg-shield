#include <stdio.h>

/* VTR Compiler Divergence PoC v2
   Undefined behavior: signed integer overflow
   GCC and Clang may optimize differently under -O2 */

int is_positive_after_increment(int x) {
    return (x + 1) > x;
}

int main(void) {
    printf("INT_MAX+1 > INT_MAX: %d\n", is_positive_after_increment(2147483647));
    printf("100 + 1 > 100: %d\n", is_positive_after_increment(100));
    return 0;
}
