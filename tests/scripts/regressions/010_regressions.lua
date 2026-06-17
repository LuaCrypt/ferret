-- expect: pass
-- category: regressions

local t = {}
for i, v in ipairs({ 2, 4, 6 }) do
    t['k' .. i] = v // 2
end
print('regressions_010', t.k1, t.k2, t.k3)
