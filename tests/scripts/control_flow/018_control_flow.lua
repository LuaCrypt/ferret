-- expect: pass
-- category: control_flow

local n = 0
repeat
    n = n + 1
until n >= 4
print('control_018', n)
