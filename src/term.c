#ifdef __linux
#include <asm-generic/ioctls.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <termios.h>
#else
#error "unsupported OS"
#endif
#include <stdio.h>

struct TermSize {
    unsigned width;
    unsigned height;
};

struct TermSize get_term_size()
{
    #ifdef __linux
    struct winsize size;
    ioctl(STDOUT_FILENO, TIOCGWINSZ, &size);
    return (struct TermSize){
        // .width = size.ws_row,
        // .height = size.ws_col
        .width = size.ws_col,
        .height = size.ws_row
    };
    #endif
}

void term_putchar(char ch)
{
    write(STDOUT_FILENO, &ch, 1);
}

void term_put_line(char *line, size_t len)
{
    write(STDOUT_FILENO, line, len);
}
