#include <stdarg.h>
#include <stdio.h>

void print_log(const char * msg, ...)
{
    va_list args;
    va_start(args, msg);
    vprintf(msg, args);
    va_end(args);
}