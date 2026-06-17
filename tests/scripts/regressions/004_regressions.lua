-- expect: pass
-- category: regressions

local total = 0
for i = 1, 5 do
    for j = 1, 3 do
        if j == 2 then break end
        total = total + i + j
    end
end
print('regressions_004', total)
