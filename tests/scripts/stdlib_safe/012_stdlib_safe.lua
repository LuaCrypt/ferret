-- expect: pass
-- category: stdlib_safe

local t = table.pack('a', 'b', '12')
print('stdlib_012', t.n, table.concat(t, ':'))
