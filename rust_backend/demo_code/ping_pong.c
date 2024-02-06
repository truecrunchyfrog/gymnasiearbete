#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int main() {
    char input[100];  // Assuming the input won't exceed 100 characters

    // Read input from stdin
    if (fgets(input, sizeof(input), stdin) == NULL) {
        perror("Error reading input");
        exit(EXIT_FAILURE);
    }

    // Remove the newline character from the input
    size_t len = strlen(input);
    if (len > 0 && input[len - 1] == '\n') {
        input[len - 1] = '\0';
    }

    // Check if the input is "ping"
    if (strcmp(input, "ping") == 0) {
        // If it is, write "pong" to stdout
        printf("pong\n");
    } else {
        // If it's not, exit with an error code
        fprintf(stderr, "Error: Unexpected input\n");
        exit(EXIT_FAILURE);
    }

    return 0;
}
