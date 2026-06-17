-- expect: pass
-- category: control_flow

local x = 3
if x % 3 == 0 then
    print('control_003', 'three')
elseif x % 2 == 0 then
    print('control_003', 'two')
else
    print('control_003', 'other')
end
