#include "common.h"

int add_nums_proxy(int a, int b) {
    return add_nums(a, b) * 3 + OFFSET_CONSTANT;
}










