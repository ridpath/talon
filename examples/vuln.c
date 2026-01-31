#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

void win() {
    printf("[!] You've reached the secret function!\n");
    system("/bin/sh");
}

void vulnerable_function() {
    char buffer[256];
    
    printf("Enter your name: ");
    fflush(stdout);
    
    gets(buffer);
    
    printf("Hello, %s!\n", buffer);
}

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin, NULL, _IONBF, 0);
    
    printf("========================================\n");
    printf("     Welcome to Vulnerable Server      \n");
    printf("========================================\n");
    printf("win() function is at: %p\n", win);
    printf("========================================\n");
    
    vulnerable_function();
    
    printf("Goodbye!\n");
    return 0;
}
