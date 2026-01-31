.PHONY: all clean vuln_binary

all: vuln_binary

vuln_binary: vuln.c
	gcc -m64 -fno-stack-protector -no-pie -z execstack -o vuln_binary vuln.c -Wno-deprecated-declarations

vuln_binary_32: vuln.c
	gcc -m32 -fno-stack-protector -no-pie -z execstack -o vuln_binary_32 vuln.c -Wno-deprecated-declarations

clean:
	rm -f vuln_binary vuln_binary_32

test: vuln_binary
	@echo "Testing vulnerable binary..."
	@echo "AAAA" | ./vuln_binary
	@echo "Build successful!"
