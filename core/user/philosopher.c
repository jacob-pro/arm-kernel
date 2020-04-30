#include "philosopher.h"
#include <string.h>

// A mod that works for negatives
int mod(int a, int b){
    return (a % b + b) %b;
}

typedef struct Philosopher {
    int id;
    int left_recv;
    int left_send;
    int right_recv;
    int right_send;
    bool has_left;
    bool has_right;
    bool reverse;
} Philosopher;

#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wmissing-noreturn"
void philosopher_function(Philosopher self) {
    char startMsg[100] = "\nPhilosopher ";
    char itoaBuffer[10];
    itoa(itoaBuffer, self.id + 1);
    strcat(startMsg, itoaBuffer);

    while (1) {
        char left_fork;
        char right_fork;

        char waitingMsg[100];
        strcpy(waitingMsg, startMsg);
        strcat(waitingMsg, " is waiting to eat");
        write(STDOUT_FILENO, waitingMsg, strlen(waitingMsg));
        if (self.reverse) {
            if (!self.has_right) read(self.right_recv, &right_fork, 1);
            if (!self.has_left) read(self.left_recv, &left_fork, 1);
        } else {
            if (!self.has_left) read(self.left_recv, &left_fork, 1);
            if (!self.has_right) read(self.right_recv, &right_fork, 1);
        }
        self.has_left = true;
        self.has_right = true;

        char eatingMsg[100];
        strcpy(eatingMsg, startMsg);
        strcat(eatingMsg, " is now eating");
        write(STDOUT_FILENO, eatingMsg, strlen(eatingMsg));

        // Eating for some time

        // Put down forks
        char finishMsg[100];
        strcpy(finishMsg, startMsg);
        strcat(finishMsg, " is finished eating");
        write(STDOUT_FILENO, finishMsg, strlen(finishMsg));
        write(self.left_send, &left_fork, 1);
        write(self.right_send, &right_fork, 1);

        self.has_left = false;
        self.has_right = false;
    }
}
#pragma clang diagnostic pop

void main_philosopher() {

    const int PHILOSOPHERS_COUNT = 16;

    int left_pipe[PHILOSOPHERS_COUNT][2];
    int right_pipe[PHILOSOPHERS_COUNT][2];

    for (int i = 0; i < PHILOSOPHERS_COUNT; i++) {
        pipe(left_pipe[i]);
        pipe(right_pipe[i]);
    }

    for (int i = 0; i < PHILOSOPHERS_COUNT; i++) {

        Philosopher x;
        x.id = i;

        int left_id = mod(i - 1, PHILOSOPHERS_COUNT);
        x.left_recv = right_pipe[left_id][0];
        x.left_send = left_pipe[i][1];

        int right_id = mod (i + 1, PHILOSOPHERS_COUNT);
        x.right_recv = left_pipe[right_id][0];
        x.right_send = right_pipe[i][1];

        if (i == 0) {       // First starts with Left + Right
            x.has_left = true;
            x.has_right = true;
            x.reverse = false;
        } else if (i == PHILOSOPHERS_COUNT - 1) {   // Last starts with none
            x.has_left = false;
            x.has_right = false;
            x.reverse = true;
        } else {        // Everyone else starts with their Right only
            x.has_left = false;
            x.has_right = true;
            x.reverse = false;
        }

        // Begin philosopher processes
        if (fork() != 0) {
            philosopher_function(x);
            break;
        }
    }

    // Don't exit so child processes stay alive
    while (1) {
        yield();
    }
    exit(EXIT_SUCCESS);
}
