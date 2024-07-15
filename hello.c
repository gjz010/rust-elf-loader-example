#include <stdio.h>
int main(int argc, char** argv){
    printf("Hello, world!\n");
    for(int i=0; i<argc; i++){
        printf("argv[%d] = %s\n", argc, argv[i]);
    }
    return 0;
}
