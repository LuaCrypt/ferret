-- expect: pass
-- category: control_flow

local total = 0
for n = 1, 7 do
    total = total + n
end
print('control_009', total)
