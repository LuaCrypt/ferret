-- expect: pass
-- category: control_flow

local n = 0
while true do
    n = n + 1
    if n == 4 then
        break
    end
end
print('control_004', n)
