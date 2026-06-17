-- expect: pass
-- category: control_flow

local x = 15
if x % 3 == 0 then
    print('control_015', 'three')
elseif x % 2 == 0 then
    print('control_015', 'two')
else
    print('control_015', 'other')
end
