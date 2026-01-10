let game = process_attach("UnityGame.exe")
let pid = game["pid"]

print("Attached to", game["name"], "PID:", pid)

let modules = process_modules(pid)
let game_base = modules[0]["base"]
print("Game base address:", hex(game_base))

let acs = anticheat_detect()
if len(acs) > 0
    print("Anti-cheat detected! Applying evasions...")
    let evasions = debugger_evasion()
    for evasion in evasions
        print("Applied:", evasion)
    end
end

let players = unity_find_objects("PlayerController")
print("Found", len(players), "player objects")

for player in players
    print("Player:", player["name"], "at", hex(player["address"]))
    
    let health_comp = unity_get_component(player["address"], "HealthComponent")
    
    let health_addr = health_comp["address"] + 0x10
    mem_write(pid, health_addr, p32(9999))
    print("Set health to 9999")
    
    let ammo_addr = health_comp["address"] + 0x20
    mem_write(pid, ammo_addr, p32(999))
    print("Set ammo to 999")
end

esp_create(pid, game_base + 0x2A3C10)

print("Game hack complete! ESP enabled.")
