#include <stdio.h>
#include <stdlib.h>
int main(int argc, char** argv){
    printf("Hello, world!\n");
    for(int i=0; i<argc; i++){
        printf("argv[%d] = %s\n", argc, argv[i]);
    }
    FILE* f = fopen("test.txt", "w");
    const char* s = "Hello, world!\n";
    fwrite(s, sizeof(char), 14, f);
    fclose(f);
    return 0;
}
