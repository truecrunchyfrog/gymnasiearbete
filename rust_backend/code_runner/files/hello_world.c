#include <stdio.h>
#include <time.h>

int main() {
    int seconds = 0;

    while (1) {
        printf("Seconds: %d\n", seconds);
        seconds++;
        sleep(1); // Wait for 1 second
        if(seconds == 12) break; // Break the loop after 10 seconds (10 iterations
    }

    return 0;
}
