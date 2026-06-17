-- expect: pass
-- category: control_flow

local x = 19
if x % 3 == 0 then
    print('control_019', 'three')
elseif x % 2 == 0 then
    print('control_019', 'two')
else
    print('control_019', 'other')
end
