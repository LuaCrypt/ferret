-- expect: pass
-- category: control_flow

local total = 0
for n = 1, 3 do
    total = total + n
end
print('control_005', total)
