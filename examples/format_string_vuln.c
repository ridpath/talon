#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

void win() {
    printf("[!] Congratulations! You've exploited the format string vulnerability!\n");
    system("/bin/sh");
}

void print_banner() {
    printf("========================================\n");
    printf("   Format String Challenge Server      \n");
    printf("========================================\n");
    printf("[DEBUG] Useful addresses:\n");
    printf("  win()    @ %p\n", win);
    printf("  printf() @ %p\n", printf);
    printf("  exit()   @ %p\n", exit);
    printf("========================================\n");
}

void vulnerable_echo() {
    char buffer[512];
    
    printf("Enter your message: ");
    fflush(stdout);
    
    if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
        return;
    }
    
    buffer[strcspn(buffer, "\n")] = '\0';
    
    printf("You said: ");
    printf(buffer);
    printf("\n");
    fflush(stdout);
}

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin, NULL, _IONBF, 0);
    
    print_banner();
    
    while (1) {
        vulnerable_echo();
        
        char choice[16];
        printf("\nContinue? (y/n): ");
        fflush(stdout);
        
        if (fgets(choice, sizeof(choice), stdin) == NULL) {
            break;
        }
        
        if (choice[0] == 'n' || choice[0] == 'N') {
            printf("Goodbye!\n");
            exit(0);
        }
    }
    
    return 0;
}
