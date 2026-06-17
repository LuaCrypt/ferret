-- expect: pass
-- category: control_flow

local total = 0
for n = 1, 5 do
    total = total + n
end
print('control_017', total)
