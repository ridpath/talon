let game = process_attach("FPS_Game.exe")
let pid = game["pid"]

print("Attached to FPS game, PID:", pid)

let modules = process_modules(pid)
let client_dll_base = 0
for mod in modules
    if mod["name"] == "client.dll"
        client_dll_base = mod["base"]
        break
    end
end

print("client.dll base:", hex(client_dll_base))

let entity_list_offset = 0x4D52D8C
let entity_list_addr = client_dll_base + entity_list_offset

esp_create(pid, entity_list_addr)
print("ESP enabled")

let entities = entity_iterate(pid, entity_list_addr)
print("Found", len(entities), "entities")

let camera_pos = [0, 0, 0]

for entity in entities
    let entity_addr = entity["address"]
    
    let visible = visibility_check(pid, entity_addr)
    
    if visible == "visible"
        let target_pos = [entity["x"], entity["y"], entity["z"]]
        
        let angles = aimbot_calculate(camera_pos, target_pos)
        
        print("Target at:", target_pos)
        print("Aim angles - Pitch:", angles["pitch"], "Yaw:", angles["yaw"])
    end
end

let view_matrix_addr = client_dll_base + 0x4D4F90
let view_matrix = []

for entity in entities
    let world_pos = [entity["x"], entity["y"], entity["z"]]
    let screen = world_to_screen(world_pos, view_matrix)
    print("Screen pos:", screen["x"], screen["y"])
end

print("Game hack complete!")
