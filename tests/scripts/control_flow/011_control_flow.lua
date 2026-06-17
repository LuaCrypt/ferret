-- expect: pass
-- category: control_flow

local x = 11
if x % 3 == 0 then
    print('control_011', 'three')
elseif x % 2 == 0 then
    print('control_011', 'two')
else
    print('control_011', 'other')
end
