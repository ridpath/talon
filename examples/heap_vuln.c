#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define MAX_CHUNKS 16
#define MAX_SIZE 0x1000

typedef struct {
    char *data;
    size_t size;
    int in_use;
} chunk_t;

chunk_t chunks[MAX_CHUNKS];

void print_banner() {
    printf("========================================\n");
    printf("     Heap Challenge - Tcache Era       \n");
    printf("========================================\n");
    printf("[*] Modern heap exploitation challenge\n");
    printf("[*] Target: glibc 2.27+ with tcache\n");
    printf("========================================\n");
}

void print_menu() {
    printf("\n--- Menu ---\n");
    printf("1. Allocate chunk\n");
    printf("2. Free chunk\n");
    printf("3. View chunk\n");
    printf("4. Edit chunk\n");
    printf("5. Exit\n");
    printf("Choice: ");
    fflush(stdout);
}

int read_int() {
    char buffer[32];
    if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
        return -1;
    }
    return atoi(buffer);
}

void allocate_chunk() {
    printf("Index (0-%d): ", MAX_CHUNKS - 1);
    fflush(stdout);
    int idx = read_int();
    
    if (idx < 0 || idx >= MAX_CHUNKS) {
        printf("[-] Invalid index\n");
        return;
    }
    
    if (chunks[idx].in_use) {
        printf("[-] Chunk already in use\n");
        return;
    }
    
    printf("Size: ");
    fflush(stdout);
    size_t size = read_int();
    
    if (size <= 0 || size > MAX_SIZE) {
        printf("[-] Invalid size\n");
        return;
    }
    
    chunks[idx].data = malloc(size);
    if (!chunks[idx].data) {
        printf("[-] Allocation failed\n");
        return;
    }
    
    chunks[idx].size = size;
    chunks[idx].in_use = 1;
    
    printf("[+] Allocated chunk at %p (size: 0x%lx)\n", chunks[idx].data, size);
    
    printf("Data (optional): ");
    fflush(stdout);
    char buffer[MAX_SIZE];
    if (fgets(buffer, sizeof(buffer), stdin) != NULL) {
        size_t len = strlen(buffer);
        if (len > 0 && buffer[len - 1] == '\n') {
            buffer[len - 1] = '\0';
        }
        strncpy(chunks[idx].data, buffer, size - 1);
        chunks[idx].data[size - 1] = '\0';
    }
}

void free_chunk() {
    printf("Index: ");
    fflush(stdout);
    int idx = read_int();
    
    if (idx < 0 || idx >= MAX_CHUNKS) {
        printf("[-] Invalid index\n");
        return;
    }
    
    if (!chunks[idx].in_use) {
        printf("[-] Chunk not in use\n");
        return;
    }
    
    free(chunks[idx].data);
    printf("[+] Freed chunk %d\n", idx);
}

void view_chunk() {
    printf("Index: ");
    fflush(stdout);
    int idx = read_int();
    
    if (idx < 0 || idx >= MAX_CHUNKS) {
        printf("[-] Invalid index\n");
        return;
    }
    
    if (!chunks[idx].in_use) {
        printf("[-] Chunk not in use\n");
        return;
    }
    
    printf("[*] Chunk %d:\n", idx);
    printf("    Address: %p\n", chunks[idx].data);
    printf("    Size: 0x%lx\n", chunks[idx].size);
    printf("    Data: %s\n", chunks[idx].data);
    
    printf("    Raw bytes (first 32): ");
    for (int i = 0; i < 32 && i < chunks[idx].size; i++) {
        printf("%02x ", (unsigned char)chunks[idx].data[i]);
    }
    printf("\n");
}

void edit_chunk() {
    printf("Index: ");
    fflush(stdout);
    int idx = read_int();
    
    if (idx < 0 || idx >= MAX_CHUNKS) {
        printf("[-] Invalid index\n");
        return;
    }
    
    if (!chunks[idx].in_use) {
        printf("[-] Chunk not in use\n");
        return;
    }
    
    printf("New data: ");
    fflush(stdout);
    
    char buffer[MAX_SIZE];
    if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
        return;
    }
    
    size_t len = strlen(buffer);
    if (len > 0 && buffer[len - 1] == '\n') {
        buffer[len - 1] = '\0';
    }
    
    strcpy(chunks[idx].data, buffer);
    
    printf("[+] Chunk updated\n");
}

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    setvbuf(stdin, NULL, _IONBF, 0);
    
    memset(chunks, 0, sizeof(chunks));
    
    print_banner();
    
    while (1) {
        print_menu();
        int choice = read_int();
        
        switch (choice) {
            case 1:
                allocate_chunk();
                break;
            case 2:
                free_chunk();
                break;
            case 3:
                view_chunk();
                break;
            case 4:
                edit_chunk();
                break;
            case 5:
                printf("[*] Exiting...\n");
                exit(0);
            default:
                printf("[-] Invalid choice\n");
        }
    }
    
    return 0;
}
