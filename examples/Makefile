.PHONY: all clean test

all: vuln_binary format_string_vuln heap_vuln

vuln_binary: vuln.c
	gcc -m64 -fno-stack-protector -no-pie -z execstack -o vuln_binary vuln.c -Wno-deprecated-declarations

vuln_binary_32: vuln.c
	gcc -m32 -fno-stack-protector -no-pie -z execstack -o vuln_binary_32 vuln.c -Wno-deprecated-declarations

format_string_vuln: format_string_vuln.c
	gcc -m64 -fno-stack-protector -no-pie -z execstack -o format_string_vuln format_string_vuln.c -Wno-deprecated-declarations

heap_vuln: heap_vuln.c
	gcc -m64 -fno-stack-protector -no-pie -z execstack -o heap_vuln heap_vuln.c -Wno-deprecated-declarations

clean:
	rm -f vuln_binary vuln_binary_32 format_string_vuln heap_vuln

test: all
	@echo "[*] Testing vulnerable binaries..."
	@echo "[+] Testing vuln_binary..."
	@echo "AAAA" | ./vuln_binary || true
	@echo "[+] Testing format_string_vuln..."
	@echo "test" | ./format_string_vuln || true
	@echo "[+] Testing heap_vuln..."
	@echo "5" | ./heap_vuln || true
	@echo "[+] All binaries built successfully!"
