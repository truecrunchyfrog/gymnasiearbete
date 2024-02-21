
#include <stdio.h>
#include <stdlib.h>

#define FILE_SIZE (1 * 1024 * 1024) // 1MB

int main(void) {
    const char *file_path = "/example_file.txt";
    FILE *file = fopen(file_path, "wb");

    if (file == NULL) {
        perror("Error opening file");
        exit(EXIT_FAILURE);
    }

    // Write 1MB of data to the file
    char buffer[1024];
    for (size_t i = 0; i < FILE_SIZE / sizeof(buffer); ++i) {
        fwrite(buffer, sizeof(buffer), 1, file);
    }

    fclose(file);

    printf("File '%s' created with size 1MB in the root directory.\n", file_path);

    return EXIT_SUCCESS;
}

