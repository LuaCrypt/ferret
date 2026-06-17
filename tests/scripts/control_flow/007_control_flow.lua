-- expect: pass
-- category: control_flow

local x = 7
if x % 3 == 0 then
    print('control_007', 'three')
elseif x % 2 == 0 then
    print('control_007', 'two')
else
    print('control_007', 'other')
end
