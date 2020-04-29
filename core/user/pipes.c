#include "pipes.h"
#include <string.h>

int writestr(int fd, char* string) {
    return write(fd, string, strlen(string));
}

void main_pipes() {

    writestr(STDOUT_FILENO, "\nStarting pipes test program");

    int ends[2] = {0, 0};
    pipe(ends);
    int read_fid = ends[0];
    int write_fid = ends[1];

    char read_fid_str[10];
    itoa(read_fid_str, read_fid);
    writestr(STDOUT_FILENO, "\nRead FID: ");
    writestr(STDOUT_FILENO, read_fid_str);

    char write_fid_str[10];
    itoa(write_fid_str, write_fid);
    writestr(STDOUT_FILENO, "\nWrite FID: ");
    writestr(STDOUT_FILENO, write_fid_str);

    char input[4] = {'T', 'E', 'S', 'T'};
    writestr(STDOUT_FILENO, "\nWriting to pipe: ");
    write(STDOUT_FILENO, input, 4);
    write(write_fid, input, 4);

    char buffer[4];
    read(read_fid, buffer, 4);
    writestr(STDOUT_FILENO, "\nRead from pipe: ");
    write(STDOUT_FILENO, buffer, 4);

    writestr(STDOUT_FILENO, "\nClosing write end");
    close(write_fid);
    int write_result = write(write_fid, input, 4);
    char write_result_str[10];
    itoa(write_result_str, write_result);
    writestr(STDOUT_FILENO, "\nAttempting to write to closed pipe returned: ");
    writestr(STDOUT_FILENO, write_result_str);

    writestr(STDOUT_FILENO, "\nAttempting to read more bytes will cause a deadlock...\n");
    read(read_fid, buffer, 1);

    exit(EXIT_SUCCESS);
}
